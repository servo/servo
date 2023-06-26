let channel;
let port;
onmessage = (e) => {
    if (e.data.port) {
        port = e.data.port;
        port.onmessage = (event) => channel.send(event.data.message);
    }
    if (e.data.channel) {
        channel = e.data.channel;
        channel.onopen = () => port.postMessage("opened");
        channel.onerror = () => port.postMessage("errored");
        channel.onclose = () => port.postMessage("closed");
        channel.onmessage = (event) => port.postMessage(event.data);
    }
};
