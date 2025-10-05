#![cfg_attr(not(feature = "std"), no_std)]

/// A pallet to enable cross-chain NFT transfers using XCM
pub use pallet::*;

pub mod xcm_handler;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{tokens::nonfungibles::Inspect, Get},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use xcm::v3::{prelude::*, MultiLocation, SendXcm};
	use xcm_executor::traits::TransactAsset;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The NFT collection ID type
		type CollectionId: Parameter + Member + Copy + MaybeSerializeDeserialize + Debug;
		/// The NFT ID type
		type ItemId: Parameter + Member + Copy + MaybeSerializeDeserialize + Debug;
		/// The origin that is allowed to send cross-chain messages
		type SendOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The XCM message sender
		type XcmSender: SendXcm;
		/// The asset transactor to handle NFT operations
		type AssetTransactor: TransactAsset;
		/// The pallet ID for this pallet
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An NFT has been sent for cross-chain transfer
		NFTSent {
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			dest_para_id: u32,
		},
		/// An NFT has been received from another chain
		NFTReceived {
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			from_para_id: u32,
		},
		/// An NFT transfer has been completed
		NFTTransferCompleted {
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			from_para_id: u32,
			to_para_id: u32,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The NFT does not exist
		NFTNotFound,
		/// The NFT is not owned by the sender
		NotOwner,
		/// Failed to send XCM message
		FailedToSendXCM,
		/// Invalid destination parachain
		InvalidDestination,
		/// Metadata exceeds maximum length
		MetadataTooLong,
	}

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	/// Map of (collection_id, item_id) to owner
	pub type NFTOwners<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::CollectionId,
		Blake2_128Concat,
		T::ItemId,
		T::AccountId,
		OptionQuery,
	>;

	/// Storage to track pending cross-chain transfers
	#[pallet::storage]
	#[pallet::getter(fn pending_transfer)]
	pub type PendingTransfers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::CollectionId,
		Blake2_128Concat,
		T::ItemId,
		MultiLocation,
		OptionQuery,
	>;

	/// Storage to preserve NFT metadata during transfers
	#[pallet::storage]
	#[pallet::getter(fn nft_metadata)]
	pub type NFTMetadata<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::CollectionId,
		Blake2_128Concat,
		T::ItemId,
		Vec<u8>, // Raw metadata bytes
		OptionQuery,
	>;

	/// Storage for NFT metadata URIs (for IPFS or other decentralized storage)
	#[pallet::storage]
	#[pallet::getter(fn nft_metadata_uri)]
	pub type NFTMetadataUri<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::CollectionId,
		Blake2_128Concat,
		T::ItemId,
		Vec<u8>, // URI as bytes (e.g., IPFS hash)
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Send an NFT to another parachain
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn send_nft(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			dest_para_id: u32,
			metadata: Vec<u8>,
			metadata_uri: Option<Vec<u8>>, // Optional URI for decentralized storage
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			
			// Call the XCM handler to process the transfer, with metadata preservation
			Self::do_xcm_transfer_nft(sender, collection_id, item_id, dest_para_id, metadata, metadata_uri)
		}

		/// Receive an NFT from another parachain - typically called by XCM execution
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn receive_nft(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			from_para_id: u32,
			owner: T::AccountId,
			metadata: Vec<u8>,
			metadata_uri: Option<Vec<u8>>, // Optional URI for decentralized storage
		) -> DispatchResult {
			// In a real implementation, this would likely be called by the XCM executor 
			// with proper origin verification, or through a privileged function
			T::SendOrigin::ensure_origin(origin)?;
			
			// Call internal function to process the receipt with metadata preservation
			Self::do_receive_nft(collection_id, item_id, from_para_id, owner, metadata, metadata_uri)
		}
		
		/// Lock an NFT for cross-chain transfer (internal function)
		pub fn lock_nft(
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			who: &T::AccountId,
		) -> DispatchResult {
			// Verify the sender owns the NFT
			let owner = Self::owner(collection_id, item_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(&owner == who, Error::<T>::NotOwner);

			// Lock the NFT by removing from active ownership but storing in pending transfers
			NFTOwners::<T>::remove(collection_id, item_id);

			// In a real implementation, we might store additional information about the lock
			Ok(())
		}
		
		/// Unlock an NFT after failed cross-chain transfer (internal function)
		pub fn unlock_nft(
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			recipient: &T::AccountId,
		) -> DispatchResult {
			// Check if this NFT is in pending transfer state
			ensure!(PendingTransfers::<T>::contains_key(collection_id, item_id), Error::<T>::NFTNotFound);

			// Restore ownership
			NFTOwners::<T>::insert(collection_id, item_id, recipient.clone());

			// Remove from pending transfers
			PendingTransfers::<T>::remove(collection_id, item_id);

			// Also clean up any associated metadata
			NFTMetadata::<T>::remove(collection_id, item_id);
			NFTMetadataUri::<T>::remove(collection_id, item_id);

			Ok(())
		}
	}

	// Implementation for handling NFT operations
	impl<T: Config> Pallet<T> {
		/// Check if an account owns a specific NFT
		pub fn is_owner(collection_id: T::CollectionId, item_id: T::ItemId, who: &T::AccountId) -> bool {
			if let Some(owner) = Self::owner(collection_id, item_id) {
				&owner == who
			} else {
				false
			}
		}

		/// Get the owner of an NFT
		pub fn get_owner(collection_id: T::CollectionId, item_id: T::ItemId) -> Option<T::AccountId> {
			Self::owner(collection_id, item_id)
		}
	}
}