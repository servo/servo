function makeSharedBuffer(size) {
  // SharedArrayBuffer constructor is hidden in some origins, but it's still
  // available via WebAssembly.Memory.
  const kPageSize = 65536;
  const sizeInPages = Math.floor((size + kPageSize - 1) / kPageSize);
  const memory = new WebAssembly.Memory(
      {initial: sizeInPages, maximum: sizeInPages, shared: true});
  return memory.buffer;
}

function runCopyToTest(frame, desc) {
  let isDone = false;
  let size = frame.allocationSize();
  let buf = new makeSharedBuffer(size);

  function runTest() {
    let startTime = PerfTestRunner.now();
    PerfTestRunner.addRunTestStartMarker();
    frame.copyTo(buf)
        .then(layout => {
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

function runBatchCopyToTest(frames, desc) {
  let isDone = false;
  let frames_and_buffers = frames.map(frame => {
    let size = frame.allocationSize();
    let buf = new makeSharedBuffer(size);
    return [frame, buf];
  });

  function runTest() {
    let startTime = PerfTestRunner.now();
    PerfTestRunner.addRunTestStartMarker();
    let readback_promises = frames_and_buffers.map(([frame, buf]) => {
      return frame.copyTo(buf);
    });
    Promise.all(readback_promises)
        .then(layouts => {
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
      for (let frame of frames)
        frame.close();
    },
    run: _ => {
      runTest();
    },
  });
}