// UI Elements
const statusEl = document.getElementById('status');
const logEl = document.getElementById('log');
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');

// WebSocket Setup
// Dynamically build the WebSocket URL 
const wsUrl = `ws://${window.location.host}/ws`;

let socket;
let reconnectTimer;

// Logging Helper
function log(message, isError = false) {
    const time = new Date().toLocaleTimeString();
    const prefix = isError ? '[ERROR]' : '[INFO]';
    logEl.textContent += `${time} ${prefix} ${message}\n`;
   
    // Auto-scroll to the bottom
    logEl.scrollTop = logEl.scrollHeight;
}

// WebSocket Connection Logic
function connect() {
    log(`Connecting to ${wsUrl}...`);
    socket = new WebSocket(wsUrl);

    // Event: Connection Opened
    socket.onopen = () => {
        log('WebSocket connection established.');
        statusEl.textContent = 'Connected';
        statusEl.className = 'status connected';
        messageInput.disabled = false;
        sendButton.disabled = false;
        // Clear any pending reconnect timer
        if (reconnectTimer) {
            clearTimeout(reconnectTimer);
            reconnectTimer = null;
        }
    };

    // Event: Message Received from ESP32
    socket.onmessage = (event) => {
        log(`Received: ${event.data}`);
    };

    // Event: Connection Closed
    socket.onclose = (event) => {
        log(`WebSocket connection closed (Code: ${event.code}). Attempting to reconnect...`, true);
        statusEl.textContent = 'Disconnected';
        statusEl.className = 'status disconnected';
        messageInput.disabled = true;
        sendButton.disabled = true;
        // Schedule a reconnection attempt in 3 seconds
        reconnectTimer = setTimeout(connect, 3000);
    };

    // Event: Error Occurred
    socket.onerror = (error) => {
        // The 'onerror' handler is usually followed by 'onclose'.
        // We'll log the error, and let 'onclose' handle the reconnect.
        log(`WebSocket error: ${error.message || 'Unknown error'}`, true);
    };
}

// Send Message Logic
function sendMessage() {
    const message = messageInput.value;
    if (message && socket && socket.readyState === WebSocket.OPEN) {
        socket.send(message);
        log(`Sent: ${message}`);
        messageInput.value = ''; // Clear the input box
    } else {
        log('Cannot send message: WebSocket is not open.', true);
    }
}

// Event Listeners for UI
sendButton.addEventListener('click', sendMessage);
messageInput.addEventListener('keypress', (event) => {
    if (event.key === 'Enter') {
        sendMessage();
    }
});

// Start the Connection on Page Load
window.addEventListener('load', connect);