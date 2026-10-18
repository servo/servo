// Send all requests through the HTTP test server, which also acts as an
// HTTPS proxy. This makes requests to the default HTTPS port, like the
// well-known file at https://<site>/.well-known/web-identity, reach the
// test server.
function FindProxyForURL(url, host) {
    return "PROXY 127.0.0.1:{{ports[http][0]}}";
}
