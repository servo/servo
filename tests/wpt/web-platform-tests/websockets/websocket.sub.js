var __SERVER__NAME = "{{host}}";
var __PORT = {{ports[ws][0]}};
var __SECURE__PORT = {{ports[wss][0]}};
var __NEW__PORT = __PORT; //All ports are non-default for now
var __NEW__SECURE__PORT = __SECURE__PORT; //All ports are non-default for now
var __PATH = "echo";
var wsocket;
var data;

function IsWebSocket() {
    if (!self.WebSocket) {
        assert_true(false, "Browser does not support WebSocket");
    }
}

function CreateWebSocketNonAbsolute() {
    IsWebSocket();
    var url = __SERVER__NAME;
    wsocket = new WebSocket(url);
}

function CreateWebSocketNonWsScheme() {
    IsWebSocket();
    var url = "http://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(url);
}

function CreateWebSocketNonAsciiProtocol(nonAsciiProtocol) {
    IsWebSocket();
    var url = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(url, nonAsciiProtocol);
}

function CreateWebSocketWithAsciiSep(asciiWithSep) {
    IsWebSocket();
    var url = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(url, asciiWithSep);
}

function CreateWebSocketWithBlockedPort(blockedPort) {
    IsWebSocket();
    var url = "ws://" + __SERVER__NAME + ":" + blockedPort + "/" + __PATH;
    return new WebSocket(url);
}

function CreateWebSocketWithSpaceInUrl(urlWithSpace) {
    IsWebSocket();
    var url = "ws://" + urlWithSpace + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(url);
}

function CreateWebSocketWithSpaceInProtocol(protocolWithSpace) {
    IsWebSocket();
    var url = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(url, protocolWithSpace);
}

function CreateWebSocketWithRepeatedProtocols() {
    IsWebSocket();
    var url = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(url, ["echo", "echo"]);
}

function CreateWebSocketWithRepeatedProtocolsCaseInsensitive() {
    IsWebSocket();
    var url = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(url, ["echo", "eCho"]);
}

function CreateWebSocket(isSecure, isProtocol, isProtocols) {
    IsWebSocket();
    var url;
    if (isSecure) {
        if (__SECURE__PORT === null) {
            throw new Error("wss not yet supported");
        }
        url = "wss://" + __SERVER__NAME + ":" + __SECURE__PORT + "/" + __PATH;
    }
    else {
        url = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    }

    if (isProtocol) {
        wsocket = new WebSocket(url, "echo");
    }
    else if (isProtocols) {
        wsocket = new WebSocket(url, ["echo", "chat"]);
    }
    else {
        wsocket = new WebSocket(url);
    }
    return wsocket;
}
