onconnect = function (event) {
    var port = event.ports[0];
    var xhr = new XMLHttpRequest;
    xhr.onerror = function () {
        port.postMessage("xhr blocked");
        port.postMessage("TEST COMPLETE");
    };
    xhr.onload = function () {
        if (xhr.responseText == "FAIL") {
            port.postMessage("xhr allowed");
        } else {
            port.postMessage("xhr blocked");
        }
        port.postMessage("TEST COMPLETE");
    };
    try {
        xhr.open("GET", "/common/redirect.py?location=http://www1.{{host}}:{{ports[http][0]}}/content-security-policy/support/fail.asis", true);
        xhr.send();
    } catch (e) {
        port.postMessage("xhr blocked");
        port.postMessage("TEST COMPLETE");
    }
}