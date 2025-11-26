console.log("Renderer process loaded");

const canvas = document.getElementById('screenCanvas');
const ctx = canvas.getContext('2d');

// Cursor overlay
const cursorOverlay = document.createElement('img');
cursorOverlay.style.position = 'absolute';
cursorOverlay.style.pointerEvents = 'none'; // Pass through clicks
cursorOverlay.style.zIndex = '1000';
cursorOverlay.style.display = 'none';
document.body.appendChild(cursorOverlay);

let videoWidth = 0;
let videoHeight = 0;

// Retry connection logic
function connect() {
    console.log("Connecting to WebSocket...");
    const ws = new WebSocket("ws://127.0.0.1:21121");
    ws.binaryType = "arraybuffer";

    ws.onopen = () => {
        console.log("WebSocket connected to ws://127.0.0.1:21121");
    };

    ws.onerror = (error) => {
        console.error("WebSocket error:", error);
    };

    ws.onclose = (event) => {
        console.log("WebSocket closed:", event.code, event.reason);
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

                // // Manually swap Red and Blue channels (BGRA -> RGBA or vice versa)
                // // Because the previous Rust-side fix might not have worked or we need to invert it.
                // // This is a fallback to ensure colors are correct.
                // const data = imageData.data;
                // for (let i = 0; i < data.length; i += 4) {
                //     const red = data[i];
                //     data[i] = data[i + 2]; // Set Red to Blue
                //     data[i + 2] = red;     // Set Blue to Red
                // }
            } else {
                // console.warn(`Data length mismatch: expected ${imageData.data.length}, got ${rawData.length}`);
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
                    // We need to convert it to an image.
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
                    // Calculate scale if canvas is displayed at different size than its resolution
                    const scaleX = rect.width / canvas.width;
                    const scaleY = rect.height / canvas.height;

                    // msg.x/y are in video coordinates.
                    const screenX = rect.left + (msg.x * scaleX) - (hotx * scaleX);
                    const screenY = rect.top + (msg.y * scaleY) - (hoty * scaleY);

                    cursorOverlay.style.left = `${screenX}px`;
                    cursorOverlay.style.top = `${screenY}px`;
                    // cursorOverlay.style.width = `${msg.width || 32}px`; // Optional
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
        // Scale factor: Video Resolution / Displayed Size
        // If videoWidth is 0, avoid division by zero
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

    // Prevent context menu on right click
    canvas.addEventListener('contextmenu', (e) => {
        e.preventDefault();
    });

    window.addEventListener('keydown', (e) => {
        sendInput({
            type: 'keydown',
            key: e.key
        });
        if (['Tab', 'Alt', 'Control', 'Meta', 'Shift'].includes(e.key)) {
            // e.preventDefault(); 
        }
    });

    window.addEventListener('keyup', (e) => {
        sendInput({
            type: 'keyup',
            key: e.key
        });
    });
}

connect();
