//! Mock implementations for XCM-related functionality

use frame_support::traits::tokens::nonfungibles::{Inspect, Transfer};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;
use xcm::v3::{prelude::*, MultiLocation, SendXcm, Xcm};
use xcm_executor::traits::TransactAsset;

// Mock implementation of an NFT interface for the bridge
pub struct MockNftHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: pallet_nft_bridge::Config> Inspect<T::AccountId> for MockNftHandler<T> {
    type ItemId = T::ItemId;
    type CollectionId = T::CollectionId;

    fn owner(
        collection_id: &Self::CollectionId,
        item_id: &Self::ItemId,
    ) -> Option<T::AccountId> {
        pallet_nft_bridge::Pallet::<T>::get_owner(*collection_id, *item_id)
    }
}

impl<T: pallet_nft_bridge::Config> Transfer<T::AccountId> for MockNftHandler<T> {
    fn transfer(
        collection_id: &Self::CollectionId,
        item_id: &Self::ItemId,
        destination: &T::AccountId,
    ) -> Result<(), DispatchError> {
        // In a real implementation, this would update ownership
        // For the bridge, we'll use our own storage system
        pallet_nft_bridge::NFTOwners::<T>::insert(collection_id, item_id, destination.clone());
        Ok(())
    }
}