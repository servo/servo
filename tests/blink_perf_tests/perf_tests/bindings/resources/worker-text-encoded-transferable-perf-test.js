const WorkerTextEncodedTransferablePerfTestRunner = (function() {
  function pingPong(data) {
    return new Promise((resolve, reject) => {
      let sendData, textEncoder, textDecoder, mainThreadBeginEncode, mainThreadEndDecode, toWorkerTime, fromWorkerTime, totalTime;
      textEncoder = new TextEncoder('utf-8');
      textDecoder  = new TextDecoder('utf-8');
      worker.addEventListener('message', function listener(e) {
        try {
          textDecoder.decode(e.data.data);
          mainThreadEndDecode = performance.now();
          worker.removeEventListener('message', listener);
          // toWorkerTime: time to encode the data, send it to the worker, and decode it on the worker
          toWorkerTime = (e.data.workerDecode + (e.data.workerTimeOrigin - performance.timeOrigin)) - mainThreadBeginEncode;
          // fromWorkerTime: time to encode the data on the worker, send it back to the main thread, and deocde it
          fromWorkerTime = mainThreadEndDecode - (e.data.workerDecode + (e.data.workerTimeOrigin - performance.timeOrigin));
          // totalTime: time to do the whole roundtrip
          totalTime = mainThreadEndDecode - mainThreadBeginEncode
          resolve([toWorkerTime, fromWorkerTime, totalTime]);
        } catch (err) { reject(err); }
      });
      mainThreadBeginEncode = performance.now();
      sendData = textEncoder.encode(data).buffer;
      worker.postMessage({"data" : sendData}, [sendData]);
    });
  }

  return {
    measureTimeAsync(test) {
      let isDone = false;
      worker = new Worker('resources/worker-text-encoded-transferable.js');
      PerfTestRunner.startMeasureValuesAsync({
        description: test.description,
        unit: 'ms',
        warmUpCount: test.warmUpCount || 10,
        iterationCount: test.iterationCount || 250,
        done() { isDone = true; },
        run: pingPongUntilDone,
      });

      function pingPongUntilDone() {
        pingPong(test.data).then(([toWorkerTime, fromWorkerTime, totalTime]) => {
          console.log([toWorkerTime, fromWorkerTime, totalTime]);
          if (test.measure == 'toWorker')
            PerfTestRunner.measureValueAsync(toWorkerTime);
          else if (test.measure === 'fromWorker')
            PerfTestRunner.measureValueAsync(fromWorkerTime);
          else if (test.measure == 'roundtrip')
            PerfTestRunner.measureValueAsync(totalTime);
          if (!isDone) pingPongUntilDone();
        });
      }
    },
  };
})();
