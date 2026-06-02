self.onmessage = function(e) {
  var data = e.data.data;
  var mainThreadEndTransfer = performance.now();
  self.postMessage({'data' : data,
                    'workerTimeOrigin' : performance.timeOrigin,
                    'mainThreadEndTransfer' : mainThreadEndTransfer},
                   [data]);
};
