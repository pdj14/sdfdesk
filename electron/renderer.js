console.log("Renderer process loaded");

const canvas = document.getElementById('screenCanvas');
const ctx = canvas.getContext('2d');

// Retry connection logic
function connect() {
    console.log("Connecting to WebSocket...");
    const ws = new WebSocket("ws://127.0.0.1:21118");
    ws.binaryType = "arraybuffer";

    ws.onopen = () => {
        console.log("WebSocket connected to ws://127.0.0.1:21118");
    };

    ws.onerror = (error) => {
        console.error("WebSocket error:", error);
    };

    ws.onclose = (event) => {
        console.log("WebSocket closed:", event.code, event.reason);
        // Optional: retry? For now, just log.
    };

    ws.onmessage = (event) => {
        const data = new DataView(event.data);

        // Parse header: width (4 bytes) + height (4 bytes)
        const width = data.getUint32(0, true); // Little endian
        const height = data.getUint32(4, true); // Little endian

        // console.log(`Received frame: ${width}x${height}, size: ${event.data.byteLength}`);

        // Resize canvas if needed
        if (canvas.width !== width || canvas.height !== height) {
            console.log(`Resizing canvas to ${width}x${height}`);
            canvas.width = width;
            canvas.height = height;
        }

        // Get raw RGBA data
        // The header is 8 bytes. The rest is pixel data.
        const rawData = new Uint8Array(event.data, 8);
        const imageData = ctx.createImageData(width, height);

        if (rawData.length === imageData.data.length) {
            imageData.data.set(rawData);
        } else {
            console.warn(`Data length mismatch: expected ${imageData.data.length}, got ${rawData.length}`);
            return;
        }

        ctx.putImageData(imageData, 0, 0);
    };

    // Input handling
    function sendInput(event) {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify(event));
        }
    }

    canvas.addEventListener('mousemove', (e) => {
        sendInput({
            type: 'mousemove',
            x: e.offsetX,
            y: e.offsetY
        });
    });

    canvas.addEventListener('mousedown', (e) => {
        let btn = 'left';
        if (e.button === 1) btn = 'middle';
        if (e.button === 2) btn = 'right';
        sendInput({
            type: 'mousedown',
            btn: btn,
            x: e.offsetX,
            y: e.offsetY
        });
    });

    canvas.addEventListener('mouseup', (e) => {
        let btn = 'left';
        if (e.button === 1) btn = 'middle';
        if (e.button === 2) btn = 'right';
        sendInput({
            type: 'mouseup',
            btn: btn,
            x: e.offsetX,
            y: e.offsetY
        });
    });

    canvas.addEventListener('wheel', (e) => {
        sendInput({
            type: 'wheel',
            delta_x: e.deltaX,
            delta_y: e.deltaY
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
