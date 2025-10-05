// Mock tests for the NFT Bridge pallet
// These tests demonstrate how the pallet would be tested in a real Substrate environment

#[cfg(test)]
mod tests {
    use crate::*;
    use frame_support::{
        assert_ok, assert_noop,
        dispatch::DispatchResult,
        parameter_types,
        traits::{ConstU32, ConstU64, Everything},
    };
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };

    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
    type Block = frame_system::mocking::MockBlock<Test>;

    frame_support::construct_runtime!(
        pub enum Test where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system,
            NftBridge: pallet_nft_bridge,
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const SS58Prefix: u8 = 42;
    }

    impl frame_system::Config for Test {
        type BaseCallFilter = Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = BlockHashCount;
        type DbWeight = ();
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = SS58Prefix;
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
    }

    // Mock configuration for the NFT Bridge pallet
    parameter_types! {
        pub const NftBridgePalletId: PalletId = PalletId(*b"nftbridg");
    }

    // Mock XCM sender that just records messages for testing
    pub struct MockXcmSender;
    impl SendXcm for MockXcmSender {
        type Ticket = ();
        fn validate(
            _destination: &mut Option<MultiLocation>,
            _message: &mut Option<Xcm<()>>,
        ) -> SendResult<Self::Ticket> {
            Ok(((), MultiAssets::new()))
        }
        fn deliver(_ticket: Self::Ticket) -> Result<XcmHash, SendError> {
            Ok([0u8; 32])
        }
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type CollectionId = u32;
        type ItemId = u32;
        type SendOrigin = frame_system::EnsureSigned<Self::AccountId>;
        type XcmSender = MockXcmSender;
        type AssetTransactor = ();
        type PalletId = NftBridgePalletId;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        t.into()
    }

    #[test]
    fn send_nft_works() {
        new_test_ext().execute_with(|| {
            let sender = 1;
            let collection_id = 1;
            let item_id = 1;
            let dest_para_id = 2000;
            let metadata = b"test_metadata".to_vec();

            // First, we need to create and assign an NFT to the sender
            // In a real test, this would be done through the NFT pallet
            NFTOwners::<Test>::insert(collection_id, item_id, sender);

            // Call the send_nft function
            assert_ok!(NftBridge::send_nft(
                RuntimeOrigin::signed(sender),
                collection_id,
                item_id,
                dest_para_id,
                metadata.clone(),
                None // no metadata URI
            ));

            // Verify that the NFT is no longer owned by the sender
            assert!(NftBridge::owner(collection_id, item_id).is_none());

            // Verify that the NFT is in pending transfer state
            assert!(NftBridge::pending_transfer(collection_id, item_id).is_some());

            // Verify that an event was emitted
            System::assert_last_event(RuntimeEvent::NftBridge(crate::Event::NFTSent {
                collection_id,
                item_id,
                dest_para_id,
            }));
        });
    }

    #[test]
    fn receive_nft_works() {
        new_test_ext().execute_with(|| {
            let collection_id = 1;
            let item_id = 1;
            let from_para_id = 2000;
            let recipient = 2;
            let metadata = b"test_metadata".to_vec();

            // Call the receive_nft function (in practice, this would be called by an authorized source)
            assert_ok!(NftBridge::receive_nft(
                RuntimeOrigin::root(), // For testing, using root as authorized origin
                collection_id,
                item_id,
                from_para_id,
                recipient,
                metadata,
                None // no metadata URI
            ));

            // Verify that the NFT is now owned by the recipient
            assert_eq!(NftBridge::owner(collection_id, item_id), Some(recipient));

            // Verify that an event was emitted
            System::assert_last_event(RuntimeEvent::NftBridge(crate::Event::NFTReceived {
                collection_id,
                item_id,
                from_para_id,
            }));
        });
    }

    #[test]
    fn send_nft_fails_if_not_owner() {
        new_test_ext().execute_with(|| {
            let sender = 1;
            let other = 2;
            let collection_id = 1;
            let item_id = 1;
            let dest_para_id = 2000;
            let metadata = b"test_metadata".to_vec();

            // Assign NFT to "other" instead of "sender"
            NFTOwners::<Test>::insert(collection_id, item_id, other);

            // Attempt to send NFT that sender doesn't own should fail
            assert_noop!(
                NftBridge::send_nft(
                    RuntimeOrigin::signed(sender),
                    collection_id,
                    item_id,
                    dest_para_id,
                    metadata,
                    None
                ),
                Error::<Test>::NotOwner
            );
        });
    }

    #[test]
    fn metadata_preservation_works() {
        new_test_ext().execute_with(|| {
            let sender = 1;
            let collection_id = 1;
            let item_id = 1;
            let dest_para_id = 2000;
            let metadata = b"test_metadata".to_vec();
            let metadata_uri = Some(b"ipfs://test".to_vec());

            // Create and assign an NFT to the sender
            NFTOwners::<Test>::insert(collection_id, item_id, sender);

            // Send the NFT with metadata
            assert_ok!(NftBridge::send_nft(
                RuntimeOrigin::signed(sender),
                collection_id,
                item_id,
                dest_para_id,
                metadata.clone(),
                metadata_uri.clone()
            ));

            // Verify that metadata is stored
            assert_eq!(NftBridge::nft_metadata(collection_id, item_id), Some(metadata));
            
            if let Some(uri) = metadata_uri {
                assert_eq!(NftBridge::nft_metadata_uri(collection_id, item_id), Some(uri));
            }
        });
    }

    #[test]
    fn nft_lock_unlock_works() {
        new_test_ext().execute_with(|| {
            let sender = 1;
            let collection_id = 1;
            let item_id = 1;

            // Create and assign an NFT to the sender
            NFTOwners::<Test>::insert(collection_id, item_id, sender);

            // Lock the NFT
            assert_ok!(NftBridge::lock_nft(collection_id, item_id, &sender));

            // Verify that the NFT is no longer owned by the sender
            assert!(NftBridge::owner(collection_id, item_id).is_none());

            // Unlock the NFT
            assert_ok!(NftBridge::unlock_nft(collection_id, item_id, &sender));

            // Verify that the NFT is owned by the sender again
            assert_eq!(NftBridge::owner(collection_id, item_id), Some(sender));
        });
    }

    #[test]
    fn nft_unlock_fails_if_not_locked() {
        new_test_ext().execute_with(|| {
            let sender = 1;
            let collection_id = 1;
            let item_id = 1;

            // Try to unlock an NFT that's not in pending transfer state
            assert_noop!(
                NftBridge::unlock_nft(collection_id, item_id, &sender),
                Error::<Test>::NFTNotFound
            );
        });
    }
}