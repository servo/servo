const WorkerStructuredCloneDifferentPayloadsPerfTestRunner = (function() {
  function pingPong(data) {
    return new Promise((resolve, reject) => {
      let mainThreadBeginSerialize, mainThreadEndDeserialize, iteration, numMessages;
      iteration = 0;
      numMessages = data['toWorker'].length;
      worker.addEventListener('message', function listener(e) {
        try {
          e.data.sendData;  // Force deserialization.
	  // keep sending messages to worker until worker runs out of responses.
	  if (!e.data.done && iteration < numMessages) {
	    iteration++;
	    worker.postMessage({'data' : data['toWorker'][iteration], 'iteration' : iteration});
	  } else {
            mainThreadEndDeserialize = performance.now();
            worker.removeEventListener('message', listener);
            totalTime = mainThreadEndDeserialize - mainThreadBeginSerialize;
            resolve([totalTime]);
	  }
        } catch (err) { reject(err); }
      });
      mainThreadBeginSerialize = performance.now();
      worker.postMessage({'data' : data['toWorker'][iteration], 'iteration' : iteration});
    });
  }

  return {
    measureTimeAsync(test) {
      let isDone = false;
      worker = new Worker('resources/worker-structured-clone-different-payloads.js');
      PerfTestRunner.startMeasureValuesAsync({
        description: test.description,
        unit: 'ms',
        warmUpCount: test.warmUpCount || 5,
        iterationCount: test.iterationCount || 15,
        done() { isDone = true; },
        run: pingPongUntilDone,
      });

      function pingPongUntilDone() {
        pingPong(test.data).then(([totalTime]) => {
          console.log([totalTime]);
          if (test.measure == 'roundtrip')
            PerfTestRunner.measureValueAsync(totalTime);
          if (!isDone) pingPongUntilDone();
        });
      }
    },
  };
})();
