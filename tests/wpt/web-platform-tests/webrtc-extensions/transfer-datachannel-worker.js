let channel;
onmessage = (event) => {
    if (event.data.channel) {
        channel = event.data.channel;
        channel.onopen = () => self.postMessage("opened");
        channel.onerror = () => self.postMessage("errored");
        channel.onclose = () => self.postMessage("closed");
        channel.onmessage = event => self.postMessage(event.data);
    }
    if (event.data.message) {
        if (channel)
            channel.send(event.data.message);
    }
    if (event.data.close) {
        if (channel)
            channel.close();
    }
};
self.postMessage("registered");
