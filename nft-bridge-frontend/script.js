// Polkadot NFT Bridge Frontend
let api;
let keyring;
let currentAccount;

// Initialize the application
async function init() {
    updateStatus("Connecting to Polkadot...", "info");
    
    try {
        // Connect to Polkadot network
        const { ApiPromise, WsProvider } = polkadot;
        
        // Using a public endpoint for demo purposes
        // In production, you'd want to let users select their preferred node
        const provider = new WsProvider('wss://rpc.polkadot.io');
        api = await ApiPromise.create({ provider });
        
        // Update connection status
        document.getElementById('status-text').textContent = 'Connected';
        document.getElementById('status-text').className = 'connected';
        
        updateStatus("Connected to Polkadot network", "success");
        
        // Enable the transfer button after connection
        document.getElementById('transfer-btn').disabled = false;
    } catch (error) {
        console.error("Failed to connect to Polkadot:", error);
        updateStatus(`Connection failed: ${error.message}`, "error");
    }
}

// Update status message
function updateStatus(message, type) {
    const statusDiv = document.getElementById('status');
    statusDiv.textContent = message;
    statusDiv.className = `status ${type}`;
    statusDiv.style.display = 'block';
    
    // Auto-hide success messages after 5 seconds
    if (type === 'success') {
        setTimeout(() => {
            statusDiv.style.display = 'none';
        }, 5000);
    }
}

// Connect to wallet
async function connectWallet() {
    try {
        // Check if polkadot-js extension is available
        if (window.injectedWeb3 && window.injectedWeb3['polkadot-js']) {
            const injected = await window.injectedWeb3['polkadot-js'].enable('NFT Bridge');
            const accounts = await injected.accounts.get();
            
            if (accounts.length > 0) {
                currentAccount = accounts[0].address;
                updateStatus(`Connected to account: ${currentAccount.substring(0, 6)}...${currentAccount.substring(currentAccount.length - 6)}`, "success");
            } else {
                updateStatus("No accounts found in wallet", "error");
            }
        } else {
            updateStatus("Polkadot.js extension not found. Please install it.", "error");
        }
    } catch (error) {
        console.error("Wallet connection error:", error);
        updateStatus(`Wallet connection failed: ${error.message}`, "error");
    }
}

// Transfer NFT function
async function transferNFT() {
    if (!currentAccount) {
        updateStatus("Please connect your wallet first", "error");
        return;
    }

    // Get form values
    const collectionId = document.getElementById('collection-id').value;
    const itemId = document.getElementById('item-id').value;
    const destParaId = document.getElementById('dest-para-id').value;
    const metadataStr = document.getElementById('metadata').value;
    const metadataUri = document.getElementById('metadata-uri').value.trim();
    
    if (!collectionId || !itemId || !destParaId) {
        updateStatus("Please fill in all required fields", "error");
        return;
    }
    
    try {
        // Validate metadata is valid JSON
        let metadata = {};
        if (metadataStr.trim()) {
            metadata = JSON.parse(metadataStr);
        }
        
        updateStatus("Preparing NFT transfer...", "info");
        
        // In a real implementation, we would interact with our custom NFT bridge pallet
        // Since we're simulating for demo purposes, we'll show what would happen
        
        // Simulate the cross-chain transfer process
        setTimeout(() => {
            updateStatus(`NFT transfer initiated: Collection ${collectionId}, Item ${itemId} to ParaID ${destParaId}`, "success");
            
            // Simulate the cross-chain process
            setTimeout(() => {
                updateStatus("NFT successfully transferred to destination chain!", "success");
            }, 3000);
        }, 1000);
        
    } catch (error) {
        console.error("Transfer error:", error);
        updateStatus(`Transfer failed: ${error.message}`, "error");
    }
}

// Event listeners
document.addEventListener('DOMContentLoaded', () => {
    // Initialize the app
    init();
    
    // Connect wallet button
    document.getElementById('connect-btn').addEventListener('click', connectWallet);
    
    // Transfer button
    document.getElementById('transfer-btn').addEventListener('click', transferNFT);
    
    // Enable transfer button only after connection
    document.getElementById('transfer-btn').disabled = true;
});