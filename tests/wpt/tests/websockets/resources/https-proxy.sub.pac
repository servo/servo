function FindProxyForURL(url, host) {
    if (dnsDomainIs(host, '.wpt')) {
        return "HTTPS {{host}}:{{ports[https][0]}}"
    }

    return "DIRECT";
}
