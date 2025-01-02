var xhr = new XMLHttpRequest;
xhr.onerror = function () {
    postMessage("xhr blocked");
    postMessage("TEST COMPLETE");
};
xhr.onload = function () {
    //cons/**/ole.log(xhr.responseText);
    if (xhr.responseText == "FAIL") {
        postMessage("xhr allowed");
    } else {
        postMessage("xhr blocked");
    }
    postMessage("TEST COMPLETE");
};
try {
    xhr.open("GET", "/common/redirect.py?location=http://www1.{{host}}:{{ports[http][0]}}/content-security-policy/support/fail.asis", true);
    xhr.send();
} catch (e) {
    postMessage("xhr blocked");
    postMessage("TEST COMPLETE");
}