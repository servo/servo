var __SERVER__NAME = "{{host}}";
var __PORT = {{ports[ws][0]}};
var __SECURE__PORT = null; //{{ports[ws][0]}}; //Should be wss
var __NEW__PORT = __PORT; //All ports are non-default for now
var __NEW_SECURE_PORT = __PORT; //All ports are non-default for now
var __PATH = "echo";
var __CONTROLPATH = "control";
var __PROTOCOL = "echo";
var __PROTOCOLS = ["echo", "chat"];
var __REPEATED__PROTOCOLS = ["echo", "echo"];
var __URL;
var __IS__WEBSOCKET;
var __PASS = "Pass";
var __FAIL = "Fail";
var wsocket;
var csocket;
var data;

// variables for testing Close Browser/Navigate Away scenarios
var isAssociated = false;
var guid;
var dataReceived;
var closeCode;
var urlToOpen;

function IsWebSocket() {
    if (!window.WebSocket) {
        assert_true(false, "Browser does not support WebSocket");
    }
}

function CreateWebSocketNonAbsolute() {
    IsWebSocket();
    __URL = __SERVER__NAME;
    wsocket = new WebSocket(__URL);
}

function CreateWebSocketNonWsScheme() {
    IsWebSocket();
    __URL = "http://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(__URL);
}

function CreateWebSocketNonAsciiProtocol(nonAsciiProtocol) {
    IsWebSocket();
    __URL = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(__URL, nonAsciiProtocol);
}

function CreateWebSocketWithBlockedPort(blockedPort) {
    IsWebSocket();
    __URL = "wss://" + __SERVER__NAME + ":" + blockedPort + "/" + __PATH;
    wsocket = new WebSocket(__URL);
}

function CreateWebSocketWithSpaceInUrl(urlWithSpace) {
    IsWebSocket();
    __URL = "ws://" + urlWithSpace + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(__URL);
}

function CreateWebSocketWithSpaceInProtocol(protocolWithSpace) {
    IsWebSocket();
    __URL = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(__URL, protocolWithSpace);
}

function CreateWebSocketWithRepeatedProtocols() {
    IsWebSocket();
    __URL = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    wsocket = new WebSocket(__URL, __REPEATED__PROTOCOLS);
}

function CreateWebSocket(isSecure, isProtocol, isProtocols) {
    IsWebSocket();
    if (isSecure) {
        if (__SECURE__PORT === null) {
            throw new Error("wss not yet supported");
        }
        __URL = "wss://" + __SERVER__NAME + ":" + __SECURE__PORT + "/" + __PATH;
    }
    else {
        __URL = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
    }

    if (isProtocol) {
        wsocket = new WebSocket(__URL, __PROTOCOL);
    }
    else if (isProtocols) {
        wsocket = new WebSocket(__URL, __PROTOCOLS);
    }
    else {
        wsocket = new WebSocket(__URL);
    }
    return wsocket;
}

function CreateControlWebSocket(isSecure) {
    IsWebSocket();
    if (isSecure) {
        __URL = "wss://" + __SERVER__NAME + ":" + __SECURE__PORT + "/" + __CONTROLPATH;
    }
    else {
        __URL = "ws://" + __SERVER__NAME + ":" + __PORT + "/" + __CONTROLPATH;
    }

    csocket = new WebSocket(__URL);
    return csocket;
}

