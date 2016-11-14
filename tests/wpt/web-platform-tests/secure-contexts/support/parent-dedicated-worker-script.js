var w = new Worker("dedicated-worker-script.js");
w.onmessage = function (e) {
  postMessage(e.data);
}
