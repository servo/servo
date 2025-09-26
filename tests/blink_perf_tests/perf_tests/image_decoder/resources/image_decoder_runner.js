function runImageDecoderPerfTests(imageFile, testDescription,
                                  alsoMeasureYUVDecoding = false) {
  var isDone = false;

  function runTest() {
    var image = new Image();

    // NB: Because it is part of the Image.Decode() specification to wait for
    // all data to be received before attempting a decode (including for frames
    // of an animated image), we must remember this metric is only for a single
    // ImageDecoder::Decode call (as opposed to progressive decoding).
    //
    // Therefore, we avoid disc read overhead and put timing as part of
    // the onload callback.
    image.onload = function() {
      PerfTestRunner.addRunTestStartMarker();
      var startTime = PerfTestRunner.now();

      // Issue a decode command
      image.decode().then(function () {
        var runTime = PerfTestRunner.now() - startTime;
        PerfTestRunner.measureValueAsync(runTime);
        PerfTestRunner.addRunTestEndMarker();

        // addRunTestEndMarker sets isDone to true once all iterations are
        // performed.
        if (!isDone) {
          // To avoid cache exhasution, ensure each run lasts at least 100ms,
          // giving our image cache (which can hold on average 6 of the images
          // used in this test) time to evict them.
          var minRunTime = 100.0;
          setTimeout(runTest, Math.max(0, minRunTime - runTime));
        }
      }).catch(error => console.log(error.message));
    }

    // Begin fetching the data
    image.src = imageFile + "?" + Math.random();
  }

  window.onload = function () {
    PerfTestRunner.startMeasureValuesAsync({
      unit: "ms",
      done: function () {
        isDone = true;
      },
      run: function () {
        runTest();
      },
      iterationCount: 20,
      description: testDescription,
      tracingCategories: 'blink',
      traceEventsToMeasure: ['ImageFrameGenerator::decode'].concat(alsoMeasureYUVDecoding ?
                                                                  ['ImageFrameGenerator::decodeToYUV'] : []),
    });
  };
}
