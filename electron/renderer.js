console.log("Renderer process loaded");

const canvas = document.getElementById('screenCanvas');
const ctx = canvas.getContext('2d');
const statusEl = document.getElementById('status');

// Cursor overlay
const cursorOverlay = document.createElement('img');
cursorOverlay.style.position = 'absolute';
cursorOverlay.style.pointerEvents = 'none'; // Pass through clicks
cursorOverlay.style.zIndex = '1000';
cursorOverlay.style.display = 'none';
document.body.appendChild(cursorOverlay);

let videoWidth = 0;
let videoHeight = 0;
let ws = null;

function updateStatus(connected) {
    if (statusEl) {
        if (connected) {
            statusEl.textContent = '● Connected';
            statusEl.className = 'status';
        } else {
            statusEl.textContent = '● Disconnected';
            statusEl.className = 'status disconnected';
        }
    }
}

// Retry connection logic
function connect() {
    console.log("Connecting to WebSocket...");
    ws = new WebSocket("ws://127.0.0.1:21121");
    ws.binaryType = "arraybuffer";

    ws.onopen = () => {
        console.log("WebSocket connected to ws://127.0.0.1:21121");
        updateStatus(true);
    };

    ws.onerror = (error) => {
        console.error("WebSocket error:", error);
        updateStatus(false);
    };

    ws.onclose = (event) => {
        console.log("WebSocket closed:", event.code, event.reason);
        updateStatus(false);
        // Optional: retry? For now, just log.
    };

    ws.onmessage = (event) => {
        const dataView = new DataView(event.data);
        const type = dataView.getUint8(0);

        if (type === 0) { // Video Frame
            // Parse header: width (4 bytes) + height (4 bytes)
            // Offset starts at 1
            const width = dataView.getUint32(1, true); // Little endian
            const height = dataView.getUint32(5, true); // Little endian

            videoWidth = width;
            videoHeight = height;

            // Resize canvas if needed
            if (canvas.width !== width || canvas.height !== height) {
                console.log(`Resizing canvas to ${width}x${height}`);
                canvas.width = width;
                canvas.height = height;
            }

            // Get raw RGBA data
            // The header is 1 (type) + 8 (dims) = 9 bytes.
            const rawData = new Uint8Array(event.data, 9);
            const imageData = ctx.createImageData(width, height);

            if (rawData.length === imageData.data.length) {
                // Copy data
                imageData.data.set(rawData);
            } else {
                // Ignore mismatch frames to avoid crash
                return;
            }

            ctx.putImageData(imageData, 0, 0);

        } else if (type === 1) { // JSON Message (Cursor)
            const textDecoder = new TextDecoder();
            const jsonText = textDecoder.decode(new Uint8Array(event.data, 1));
            try {
                const msg = JSON.parse(jsonText);
                if (msg.type === 'cursor_data') {
                    // msg.data is base64 encoded RAW PIXELS (RGBA), NOT a PNG file.
                    const binaryString = atob(msg.data);
                    const len = binaryString.length;
                    const bytes = new Uint8Array(len);
                    for (let i = 0; i < len; i++) {
                        bytes[i] = binaryString.charCodeAt(i);
                    }

                    // Create a temp canvas to generate the image
                    const tempCanvas = document.createElement('canvas');
                    tempCanvas.width = msg.width;
                    tempCanvas.height = msg.height;
                    const tempCtx = tempCanvas.getContext('2d');
                    const imgData = tempCtx.createImageData(msg.width, msg.height);

                    // Check size
                    if (bytes.length === imgData.data.length) {
                        imgData.data.set(bytes);
                        tempCtx.putImageData(imgData, 0, 0);
                        cursorOverlay.src = tempCanvas.toDataURL();
                        cursorOverlay.style.display = 'block';
                        cursorOverlay.dataset.hotx = msg.hotx;
                        cursorOverlay.dataset.hoty = msg.hoty;
                    } else {
                        console.warn("Cursor data size mismatch", bytes.length, imgData.data.length);
                    }

                } else if (msg.type === 'cursor_position') {
                    const hotx = parseInt(cursorOverlay.dataset.hotx || '0');
                    const hoty = parseInt(cursorOverlay.dataset.hoty || '0');

                    const rect = canvas.getBoundingClientRect();
                    const scaleX = rect.width / canvas.width;
                    const scaleY = rect.height / canvas.height;

                    const screenX = rect.left + (msg.x * scaleX) - (hotx * scaleX);
                    const screenY = rect.top + (msg.y * scaleY) - (hoty * scaleY);

                    cursorOverlay.style.left = `${screenX}px`;
                    cursorOverlay.style.top = `${screenY}px`;
                } else if (msg.type === 'error') {
                    console.error('Connection error:', msg);

                    const errorDiv = document.createElement('div');
                    errorDiv.style.cssText = `
                        position: fixed;
                        top: 50%;
                        left: 50%;
                        transform: translate(-50%, -50%);
                        background-color: rgba(220, 38, 38, 0.95);
                        color: white;
                        padding: 30px;
                        border-radius: 10px;
                        z-index: 10000;
                        max-width: 400px;
                        text-align: center;
                        box-shadow: 0 4px 20px rgba(0,0,0,0.5);
                        font-family: Arial, sans-serif;
                    `;

                    errorDiv.innerHTML = `
                        <h2 style="margin: 0 0 10px 0; font-size: 20px;">${msg.title || 'Error'}</h2>
                        <p style="margin: 0 0 20px 0; font-size: 16px;">${msg.message || 'An error occurred'}</p>
                        <button onclick="this.parentElement.remove()" style="
                            padding: 10px 30px;
                            font-size: 14px;
                            border: none;
                            border-radius: 5px;
                            background-color: white;
                            color: #dc2626;
                            cursor: pointer;
                            font-weight: bold;
                        ">Close</button>
                    `;

                    document.body.appendChild(errorDiv);

                    setTimeout(() => {
                        if (errorDiv.parentNode) {
                            errorDiv.remove();
                        }
                    }, 15000);
                }
            } catch (e) {
                console.error("Failed to parse JSON message", e);
            }
        }
    };

    // Input handling
    function sendInput(event) {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify(event));
        }
    }

    function getScaledCoordinates(e) {
        const rect = canvas.getBoundingClientRect();
        if (videoWidth === 0 || videoHeight === 0) return { x: Math.round(e.offsetX), y: Math.round(e.offsetY) };

        const scaleX = videoWidth / rect.width;
        const scaleY = videoHeight / rect.height;

        return {
            x: Math.round(e.offsetX * scaleX),
            y: Math.round(e.offsetY * scaleY)
        };
    }

    canvas.addEventListener('mousemove', (e) => {
        const pos = getScaledCoordinates(e);
        sendInput({
            type: 'mousemove',
            x: pos.x,
            y: pos.y
        });
    });

    canvas.addEventListener('mousedown', (e) => {
        let btn = 'left';
        if (e.button === 1) btn = 'middle';
        if (e.button === 2) btn = 'right';
        const pos = getScaledCoordinates(e);
        sendInput({
            type: 'mousedown',
            btn: btn,
            x: pos.x,
            y: pos.y
        });
    });

    canvas.addEventListener('mouseup', (e) => {
        let btn = 'left';
        if (e.button === 1) btn = 'middle';
        if (e.button === 2) btn = 'right';
        const pos = getScaledCoordinates(e);
        sendInput({
            type: 'mouseup',
            btn: btn,
            x: pos.x,
            y: pos.y
        });
    });

    canvas.addEventListener('wheel', (e) => {
        sendInput({
            type: 'wheel',
            delta_x: -e.deltaX,
            delta_y: -e.deltaY
        });
        e.preventDefault();
    }, { passive: false });

    canvas.addEventListener('contextmenu', (e) => {
        e.preventDefault();
    });

    window.addEventListener('keydown', (e) => {
        // Don't capture input field key events
        if (e.target.tagName === 'INPUT') return;

        sendInput({
            type: 'keydown',
            key: e.key
        });
    });

    window.addEventListener('keyup', (e) => {
        // Don't capture input field key events
        if (e.target.tagName === 'INPUT') return;

        sendInput({
            type: 'keyup',
            key: e.key
        });
    });

    // ========================================
    // Login Control Buttons
    // ========================================

    // Send Ctrl+Alt+Delete (SAS)
    document.getElementById('btnCAD').addEventListener('click', () => {
        console.log('Sending Ctrl+Alt+Delete...');
        sendInput({
            type: 'send_sas'
        });
    });

    // Send ID (username) - use env var if provided
    const unlockId = process.env.SDFDESK_UNLOCK_ID || '';
    const unlockPw = process.env.SDFDESK_UNLOCK_PW || '';

    // Show buttons if credentials were provided via CLI
    if (unlockId) {
        document.getElementById('dividerID').style.display = 'block';
        document.getElementById('btnSendID').style.display = 'inline-block';
        console.log('Unlock ID provided via CLI, showing Send UserId button');
    }
    if (unlockPw) {
        document.getElementById('dividerPW').style.display = 'block';
        document.getElementById('btnSendPW').style.display = 'inline-block';
        console.log('Unlock PW provided via CLI, showing Send PW button');
    }

    document.getElementById('btnSendID').addEventListener('click', () => {
        if (!unlockId) {
            console.log('No unlock ID provided');
            return;
        }
        console.log('Sending UserId from CLI...');
        sendInput({
            type: 'send_text',
            text: unlockId,
            enter: false
        });
    });

    // Send PW (password) - use env var if provided
    document.getElementById('btnSendPW').addEventListener('click', () => {
        if (!unlockPw) {
            console.log('No unlock PW provided');
            return;
        }
        console.log('Sending Password from CLI...');
        sendInput({
            type: 'send_text',
            text: unlockPw,
            enter: true  // Press Enter after password
        });
    });
}

connect();

