onconnect = function(event) {
    var port = event.ports[0];
    try {
        var xhr = new XMLHttpRequest;
        xhr.open("GET", "http://www1.{{host}}:{{ports[http][0]}}/content-security-policy/blink-contrib/resources/blue.css", true);
        port.postMessage("xhr allowed");
    } catch (e) {
        port.postMessage("xhr blocked");
    }
};
