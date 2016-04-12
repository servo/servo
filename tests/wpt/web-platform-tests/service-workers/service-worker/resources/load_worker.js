self.onmessage = function (evt) {
    if (evt.data == "xhr") {
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "synthesized-response.txt", true);
        xhr.responseType = "text";
        xhr.send();
        xhr.onload = function (evt) {
            postMessage(xhr.responseText);
        };
        xhr.onerror = function() {
            postMessage("XHR failed!");
        };
    } else if (evt.data == "fetch") {
        fetch("synthesized-response.txt")
          .then(function(response) {
              return response.text();
            })
          .then(function(data) {
              postMessage(data);
            })
          .catch(function(error) {
              postMessage("Fetch failed!");
            });
    } else if (evt.data == "importScripts") {
        importScripts("synthesized-response.js");
    } else {
        throw "Unexpected message! " + evt.data;
    }
};
