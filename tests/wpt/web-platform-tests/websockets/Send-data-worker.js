onmessage = function (evt) {
    importScripts('websocket.js?pipe=sub')
    var wsocket = CreateWebSocket(false, false, false);

    wsocket.addEventListener('open', function (e) {
        wsocket.send(evt.data)
    }, true)

    wsocket.addEventListener('message', function (e) {
        postMessage(e.data);
    }, true);
}

