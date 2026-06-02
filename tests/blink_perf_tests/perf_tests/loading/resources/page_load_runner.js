/*
  Test loading of a page and waiting for the page to load.
  PerfTestRunner.measurePageLoadTime doesn't wait for external resources
  and is geared more towards parse/layout testing.
*/
function runPageLoadPerfTest(htmlFile, testDescription) {
  let isDone = false;
  let count = 0;

  function runTest() {
    PerfTestRunner.addRunTestStartMarker();
    let startTime = PerfTestRunner.now();
    count++;
    let testFrame = document.createElement('iframe');
    testFrame.src = htmlFile + "?run=" + count;
    testFrame.width = "10";
    testFrame.height = "10";
    document.querySelector('body').appendChild(testFrame);

    testFrame.onload = function() {
      let runTime = PerfTestRunner.now() - startTime;
      PerfTestRunner.measureValueAsync(runTime);
      PerfTestRunner.addRunTestEndMarker();

      if (!isDone) {
        testFrame.remove();
        var minRunTime = 100.0;
        setTimeout(runTest, Math.max(0, minRunTime - runTime));
      }
    }
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
      iterationCount: 7,
      warmUpCount: 2,
      description: testDescription
    });
  };
}
