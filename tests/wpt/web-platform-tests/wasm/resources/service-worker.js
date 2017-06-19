var port;

importScripts('load_wasm.js');

self.onmessage = function(e) {
    var message = e.data;
    if ('port' in message) {
        port = message.port;
    }
};

// And an event listener:
self.addEventListener('message', function(e) {
    var message = e.data;
    if ("compile" in message) {
        createWasmModule()
            .then(m => {
                try {
                    port.postMessage({type:"OK", module:m});
                } catch (e) {
                    port.postMessage({type:"SEND ERROR"});
                }
            })
            .catch(e => port.postMessage({type:"OTHER ERROR"}));
    }
});
