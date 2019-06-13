self.onmessage = evt => {
  if (evt.data == "xhr") {
    const xhr = new XMLHttpRequest();
    xhr.open("GET", "synthesized-response.txt", true);
    xhr.responseType = "text";
    xhr.send();
    xhr.onload = evt => postMessage(xhr.responseText);
    xhr.onerror = () => postMessage("XHR failed!");
  } else if (evt.data == "fetch") {
    fetch("synthesized-response.txt")
        .then(response => response.text())
        .then(data => postMessage(data))
        .catch(error => postMessage("Fetch failed!"));
  } else if (evt.data == "importScripts") {
    importScripts("synthesized-response.js");
  } else {
    postMessage("Unexpected message! " + evt.data);
  }
};
