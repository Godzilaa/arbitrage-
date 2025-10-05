//! XCM integration for NFT transfers

use crate::*;
use frame_support::traits::tokens::nonfungibles::Inspect;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;
use xcm::v3::{prelude::*, MultiLocation, SendXcm, Xcm};
use xcm_executor::traits::TransactAsset;

// Implementation for XCM-based NFT operations
impl<T: Config> Pallet<T> {
	/// Execute the cross-chain transfer of an NFT using XCM
	pub fn do_xcm_transfer_nft(
		sender: T::AccountId,
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		dest_para_id: u32,
		metadata: Vec<u8>,
		metadata_uri: Option<Vec<u8>>, // Optional URI for decentralized storage
	) -> DispatchResult {
		// Verify the sender owns the NFT
		let owner = Self::owner(collection_id, item_id).ok_or(Error::<T>::NFTNotFound)?;
		ensure!(owner == sender, Error::<T>::NotOwner);

		// Validate metadata length
		ensure!(metadata.len() <= 1024, Error::<T>::MetadataTooLong);

		// Store metadata for preservation during cross-chain transfer
		NFTMetadata::<T>::insert(collection_id, item_id, metadata);
		
		if let Some(uri) = metadata_uri {
			// Store the URI for decentralized metadata access
			ensure!(uri.len() <= 256, Error::<T>::MetadataTooLong); // Limit URI length
			NFTMetadataUri::<T>::insert(collection_id, item_id, uri);
		}

		// Lock the NFT (remove from owner's possession temporarily)
		Self::lock_nft(collection_id, item_id, &sender)?;

		// Construct the destination location
		let dest_para_id_location = Parachain(dest_para_id).into();
		let dest_location = MultiLocation {
			parents: 1,
			interior: dest_para_id_location,
		};

		// Store as pending transfer
		PendingTransfers::<T>::insert(collection_id, item_id, dest_location.clone());

		// For true NFT transfers, we need to handle them as unique assets
		// This is a simplified example - in a real implementation, we'd need to work with
		// specific NFT asset classes
		let message = Xcm(vec![
			// Reserve the asset on this chain
			ReserveAssetDeposited((
				vec![MultiAsset {
					id: AssetId::Concrete(MultiLocation {
						parents: 0,
						interior: X2(
							PalletInstance(<T as frame_system::Config>::PalletInfo::index::<Self>()
								.ok_or(Error::<T>::InvalidDestination)? as u8),
							GeneralIndex(collection_id.encode().using_encoded(|b| {
								b.iter().take(8).fold(0u128, |acc, &x| (acc << 8) | x as u128)
							})),
						),
					}),
					fun: Fungibility::NonFungible(
						item_id.encode().using_encoded(|b| {
							b.iter().take(16).fold(0u128, |acc, &x| (acc << 8) | x as u128)
						}).into()
					),
				}].into(),
			).into()),
			// Clear the origin
			ClearOrigin,
			// Buy execution time on destination
			BuyExecution {
				fees: (MultiLocation { parents: 1, interior: Here }, 1_000_000_000u128).into(),
				weight_limit: Limited(Weight::from_parts(400_000_000_000, 64 * 1024)),
			},
			// Transfer and deposit on destination
			InitiateReserveWithdraw {
				assets: All.into(),
				reserve: dest_location.clone(),
				xcm: Xcm(vec![
					DepositAsset {
						assets: AllCounted(1).into(),
						beneficiary: MultiLocation {
							parents: 0,
							interior: X1(AccountId32 { 
								network: None, 
								id: sender.encode().try_into().map_err(|_| Error::<T>::FailedToSendXCM)? 
							}),
						},
					}
				]),
			},
		]);

		// Send the XCM message
		T::XcmSender::send_xcm(dest_location, message)
			.map_err(|_| Error::<T>::FailedToSendXCM)?;

		Self::deposit_event(Event::NFTSent {
			collection_id,
			item_id,
			dest_para_id,
		});

		Ok(())
	}
	
	/// Handle receipt of an NFT from another chain
	pub fn do_receive_nft(
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		from_para_id: u32,
		recipient: T::AccountId,
		metadata: Vec<u8>,
		metadata_uri: Option<Vec<u8>>, // Optional URI for decentralized storage
	) -> DispatchResult {
		// Validate metadata length
		ensure!(metadata.len() <= 1024, Error::<T>::MetadataTooLong);

		// Mint the NFT to the specified recipient
		NFTOwners::<T>::insert(collection_id, item_id, recipient.clone());

		// Store the metadata to maintain it on this chain
		NFTMetadata::<T>::insert(collection_id, item_id, metadata);
		
		if let Some(uri) = metadata_uri {
			ensure!(uri.len() <= 256, Error::<T>::MetadataTooLong); // Limit URI length
			NFTMetadataUri::<T>::insert(collection_id, item_id, uri);
		}

		// Remove from pending transfers if it exists
		PendingTransfers::<T>::remove(collection_id, item_id);

		Self::deposit_event(Event::NFTReceived {
			collection_id,
			item_id,
			from_para_id,
		});

		Ok(())
	}
}