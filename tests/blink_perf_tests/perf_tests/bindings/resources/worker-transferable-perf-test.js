const WorkerTransferablePerfTestRunner = (function() {
  function pingPong(data) {
    return new Promise((resolve, reject) => {
      let sendData, mainThreadBeginTransfer, workerEndTransfer, toWorkerTime, fromWorkerTime, totalTime;
      worker.addEventListener('message', function listener(e) {
        try {
          e.data.data;
          workerEndTransfer = performance.now();
          worker.removeEventListener('message', listener);
          // toWorkerTime: Time for the main thread to transfer data to the worker
          toWorkerTime = (e.data.mainThreadEndTransfer + (e.data.workerTimeOrigin - performance.timeOrigin)) - mainThreadBeginTransfer;
          // fromWorkerTime: Time for the worker to transfer data back to the main thread
          fromWorkerTime = workerEndTransfer - (e.data.mainThreadEndTransfer + (e.data.workerTimeOrigin - performance.timeOrigin));
          // totalTime: Time from main thread beginning transfer to the end of the worker transfer
          totalTime = workerEndTransfer - mainThreadBeginTransfer
          resolve([toWorkerTime, fromWorkerTime, totalTime]);
        } catch (err) { reject(err); }
      });
      sendData = data.slice(0);  // Copy the data for every new transfer
      mainThreadBeginTransfer = performance.now();
      worker.postMessage({"data" : sendData}, [sendData]);
    });
  }

  return {
    measureTimeAsync(test) {
      let isDone = false;
      worker = new Worker('resources/worker-transferable.js');
      PerfTestRunner.startMeasureValuesAsync({
        description: test.description,
        unit: 'ms',
        warmUpCount: test.warmUpCount || 10,
        iterationCount: test.iterationCount || 250,
        done() { isDone = true;},
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
