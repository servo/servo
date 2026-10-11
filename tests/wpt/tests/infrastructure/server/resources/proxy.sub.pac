function FindProxyForURL(url, host) {
    if (dnsDomainIs(host, '.wpt.test') || url.indexOf(":99") != -1) {
        return "PROXY 127.0.0.1:{{ports[http][0]}}"
    }

    return "DIRECT";
}
