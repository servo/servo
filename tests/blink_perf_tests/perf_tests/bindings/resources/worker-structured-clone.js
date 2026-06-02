self.onmessage = function(e) {
  var data = e.data;  // Force deserialization
  var workerDeserialize = performance.now();
  self.postMessage({'recieveddData' : data,
                    'workerTimeOrigin' : performance.timeOrigin,
                    'workerDeserialize' : workerDeserialize});
};
