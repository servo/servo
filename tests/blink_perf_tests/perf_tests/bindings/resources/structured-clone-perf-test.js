const StructuredClonePerfTestRunner = (function() {
  function pingPong(data) {
    return new Promise((resolve, reject) => {
      let beginSerialize, endSerialize, beginDeserialize, endDeserialize;
      window.addEventListener('message', function listener(e) {
        try {
          e.data;  // Force deserialization.
          endDeserialize = PerfTestRunner.now();
          window.removeEventListener('message', listener);
          resolve([endSerialize - beginSerialize, endDeserialize - beginDeserialize]);
        } catch (err) { reject(err); }
      });

      beginSerialize = PerfTestRunner.now();
      window.postMessage(data, '*');
      beginDeserialize = endSerialize = PerfTestRunner.now();
      // While Chrome does the deserialize lazily when e.data is read, this
      // isn't portable, so it's more fair to measure from when the message is
      // posted.
    });
  }

  return {
    measureTimeAsync(test) {
      let isDone = false;
      PerfTestRunner.startMeasureValuesAsync({
        description: test.description,
        unit: 'ms',
        warmUpCount: test.warmUpCount || 10,
        iterationCount: test.iterationCount || 250,
        done() { isDone = true; },
        run: pingPongUntilDone,
      });

      function pingPongUntilDone() {
        pingPong(test.data).then(([serializeTime, deserializeTime]) => {
          console.log([serializeTime, deserializeTime]);
          if (test.measure === 'serialize')
            PerfTestRunner.measureValueAsync(serializeTime);
          else if (test.measure === 'deserialize')
            PerfTestRunner.measureValueAsync(deserializeTime);
          if (!isDone) pingPongUntilDone();
        });
      }
    },
  };
})();
