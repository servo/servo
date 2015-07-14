try {
    var xhr = new XMLHttpRequest;
    xhr.open("GET", "http://127.0.0.1:8000/xmlhttprequest/resources/get.txt", true);
    postMessage("xhr allowed");
} catch (e) {
    postMessage("xhr blocked");
}
