var textDecoder = new TextDecoder('utf-8');
var textEncoder = new TextEncoder('utf-8');
self.onmessage = function(e) {
  var data = textDecoder.decode(e.data.data);
  var workerDecode = performance.now();
  var sendData = textEncoder.encode(data).buffer;
  self.postMessage({'data' : sendData,
                    'workerTimeOrigin' : performance.timeOrigin,
                    'workerDecode' : workerDecode},
                   [sendData]);
};
