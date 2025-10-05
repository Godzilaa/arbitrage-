# Polkadot NFT Bridge

A cross-chain NFT bridge for transferring NFTs between Polkadot parachains using XCM (Cross-Consensus Messaging).

## Overview

This project implements a pallet for enabling secure, trustless transfers of NFTs between parachains in the Polkadot ecosystem. The bridge preserves NFT metadata during transfers and ensures atomicity of cross-chain operations.

## Features

- Cross-chain NFT transfers using XCM
- Metadata preservation during transfers
- Lock/unlock mechanism for secure transfers
- Support for decentralized metadata storage (IPFS)
- Web-based frontend for user interactions

## Architecture

The bridge consists of:

1. **NFT Bridge Pallet**: Core logic for handling NFT transfers
2. **XCM Integration**: Handles cross-chain messaging
3. **Metadata System**: Preserves NFT metadata during transfers
4. **Frontend Interface**: Web application for user interactions

## Components

### Pallet-nft-bridge
- `send_nft`: Initiates cross-chain transfer
- `receive_nft`: Handles receipt of NFTs from other chains
- `lock_nft`/`unlock_nft`: Internal functions for secure transfer handling
- NFT ownership tracking
- Metadata preservation system

### Frontend
- Connect to Polkadot wallet
- Transfer NFTs between parachains
- View transfer status

## Usage

### For Developers
1. Include `pallet-nft-bridge` in your Substrate node
2. Configure XCM dependencies
3. Set up the bridge with appropriate permissions

### For Users
1. Connect your Polkadot.js wallet
2. Enter NFT details (collection ID, item ID)
3. Specify destination parachain
4. Include metadata and optional URI
5. Execute transfer

## Technical Details

The bridge leverages:
- XCM (Cross-Consensus Messaging) for cross-chain communication
- Substrate's pallet system for NFT operations
- Polkadot's shared security model
- Decentralized storage for metadata (IPFS support)

## Implementation Notes

This implementation:
- Uses a simplified approach to NFT representation in XCM
- Implements metadata preservation using on-chain storage
- Provides both direct metadata and metadata URI options
- Follows Substrate pallet best practices

## Future Enhancements

Potential improvements:
- Better error handling and recovery mechanisms
- Batch transfer capabilities
- Enhanced security features
- More sophisticated metadata handling
- Support for different NFT standards (RMRK, etc.)