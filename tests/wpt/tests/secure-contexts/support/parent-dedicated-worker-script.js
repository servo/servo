// If nested workers aren't supported, punt:
if (typeof Worker != "undefined") {
  var w = new Worker("dedicated-worker-script.js");
  w.onmessage = function (e) {
    postMessage(e.data);
  }
} else {
  postMessage("Nested workers not supported.");
}
