var xhr = new XMLHttpRequest();
xhr.open("GET", "data/preact-from-worker.json", false);
xhr.send(null);
var sendData = JSON.parse(xhr.responseText)
var length = sendData['fromWorker'].length;

self.onmessage = function(e) {
  var data = e.data;  // Force deserialization.
  var iteration = e.data.iteration;
  var done = false;
  if (iteration == length - 1){
    done = true;
  }
  self.postMessage({'sendData' : sendData['fromWorker'][iteration], 'done': done});
};
