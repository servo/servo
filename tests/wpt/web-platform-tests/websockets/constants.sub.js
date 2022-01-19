const __SERVER__NAME = "{{host}}";
const __PATH = "echo";

var __SCHEME;
var __PORT;
if (url_has_variant('wss')) {
  __SCHEME = 'wss';
  __PORT = "{{ports[wss][0]}}";
} else if (url_has_flag('h2')) {
  __SCHEME = 'wss';
  __PORT = "{{ports[h2][0]}}";
} else {
  __SCHEME = 'ws';
  __PORT = "{{ports[ws][0]}}";
}

const SCHEME_DOMAIN_PORT = __SCHEME + '://' + __SERVER__NAME + ':' + __PORT;

function url_has_variant(variant) {
  const params = new URLSearchParams(location.search);
  return params.get(variant) === "";
}

function url_has_flag(flag) {
  const params = new URLSearchParams(location.search);
  return params.getAll("wpt_flags").indexOf(flag) !== -1;
}

function IsWebSocket() {
  if (!self.WebSocket) {
    assert_true(false, "Browser does not support WebSocket");
  }
}

function CreateWebSocketNonAbsolute() {
  IsWebSocket();
  const url = __SERVER__NAME;
  return new WebSocket(url);
}

function CreateWebSocketNonWsScheme() {
  IsWebSocket();
  const url = "http://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH;
  return new WebSocket(url);
}

function CreateWebSocketNonAsciiProtocol(nonAsciiProtocol) {
  IsWebSocket();
  const url = SCHEME_DOMAIN_PORT + "/" + __PATH;
  return new WebSocket(url, nonAsciiProtocol);
}

function CreateWebSocketWithAsciiSep(asciiWithSep) {
  IsWebSocket();
  const url = SCHEME_DOMAIN_PORT + "/" + __PATH;
  return new WebSocket(url, asciiWithSep);
}

function CreateWebSocketWithBlockedPort(blockedPort) {
  IsWebSocket();
  const url = __SCHEME + "://" + __SERVER__NAME + ":" + blockedPort + "/" + __PATH;
  return new WebSocket(url);
}

function CreateWebSocketWithSpaceInUrl(urlWithSpace) {
  IsWebSocket();
  const url = __SCHEME + "://" + urlWithSpace + ":" + __PORT + "/" + __PATH;
  return new WebSocket(url);
}

function CreateWebSocketWithSpaceInProtocol(protocolWithSpace) {
  IsWebSocket();
  const url = SCHEME_DOMAIN_PORT + "/" + __PATH;
  return new WebSocket(url, protocolWithSpace);
}

function CreateWebSocketWithRepeatedProtocols() {
  IsWebSocket();
  const url = SCHEME_DOMAIN_PORT + "/" + __PATH;
  return new WebSocket(url, ["echo", "echo"]);
}

function CreateWebSocketWithRepeatedProtocolsCaseInsensitive() {
  IsWebSocket();
  const url = SCHEME_DOMAIN_PORT + "/" + __PATH;
  wsocket = new WebSocket(url, ["echo", "eCho"]);
}

function CreateWebSocket(isProtocol, isProtocols) {
  IsWebSocket();
  const url = SCHEME_DOMAIN_PORT + "/" + __PATH;

  if (isProtocol) {
    return new WebSocket(url, "echo");
  }
  if (isProtocols) {
    return new WebSocket(url, ["echo", "chat"]);
  }
  return new WebSocket(url);
}
