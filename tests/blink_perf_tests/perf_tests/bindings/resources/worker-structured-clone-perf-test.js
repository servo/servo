const WorkerStructuredClonePerfTestRunner = (function() {
  function pingPong(data) {
    return new Promise((resolve, reject) => {
      let mainThreadBeginSerialize, mainThreadEndDeserialize, toWorkerTime, fromWorkerTime, totalTime;
      worker.addEventListener('message', function listener(e) {
        try {
          e.data;  // Force deserialization.
          mainThreadEndDeserialize = performance.now();
          worker.removeEventListener('message', listener);
          // toWorkerTime: Time from main thread beginning serialize to end of worker deserialize
          toWorkerTime = (e.data.workerDeserialize + (e.data.workerTimeOrigin - performance.timeOrigin)) - mainThreadBeginSerialize;
          // fromWorkerTime: Time from worker beginning serialize to end of main thread deserialize
          fromWorkerTime = mainThreadEndDeserialize - (e.data.workerDeserialize + (e.data.workerTimeOrigin - performance.timeOrigin));
          // totalTime: Time from main thread beginning serialzie to end of main thread deserialize
          totalTime = mainThreadEndDeserialize - mainThreadBeginSerialize
          resolve([toWorkerTime, fromWorkerTime, totalTime]);
        } catch (err) { reject(err); }
      });
      mainThreadBeginSerialize = performance.now();
      worker.postMessage(data);
    });
  }

  return {
    measureTimeAsync(test) {
      let isDone = false;
      worker = new Worker('resources/worker-structured-clone.js');
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
