var worker = new Worker("load_worker.js");

self.onmessage = function (evt) {
  worker.postMessage(evt.data);
};

worker.onmessage = function (evt) {
  self.postMessage(evt.data);
}
