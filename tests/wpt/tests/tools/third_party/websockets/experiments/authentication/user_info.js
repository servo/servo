window.addEventListener("DOMContentLoaded", () => {
    const uri = `ws://token:${token}@localhost:8004/`;
    const websocket = new WebSocket(uri);

    websocket.onmessage = ({ data }) => {
        // event.data is expected to be "Hello <user>!"
        websocket.send(`Goodbye ${data.slice(6, -1)}.`);
    };

    runTest(websocket);
});
