function runCreateImageBitmapTest(frame, desc) {
  let isDone = false;

  function runTest() {
    let startTime = PerfTestRunner.now();
    PerfTestRunner.addRunTestStartMarker();
    window.createImageBitmap(frame)
        .then(bitmap => {
          PerfTestRunner.measureValueAsync(PerfTestRunner.now() - startTime);
          PerfTestRunner.addRunTestEndMarker();
          if (!isDone)
            runTest();
        })
        .catch(e => {
          PerfTestRunner.logFatalError('Test error: ' + e);
        })
  }

  PerfTestRunner.startMeasureValuesAsync({
    description: desc,
    unit: 'ms',
    done: _ => {
      isDone = true;
      frame.close();
    },
    run: _ => {
      runTest();
    },
  });
}
