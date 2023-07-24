// Utilities for Layout Instability tests.

// Returns a promise that is resolved when the specified number of animation
// frames has occurred.
waitForAnimationFrames = frameCount => {
  return new Promise(resolve => {
    const handleFrame = () => {
      if (--frameCount <= 0)
        resolve();
      else
        requestAnimationFrame(handleFrame);
    };
    requestAnimationFrame(handleFrame);
  });
};

// Returns a promise that is resolved when the next animation frame occurs.
waitForAnimationFrame = () => waitForAnimationFrames(1);

// Helper to compute an expected layout shift score based on an expected impact
// region and max move distance for a particular animation frame.
computeExpectedScore = (impactRegionArea, moveDistance) => {
  const docElement = document.documentElement;

  const viewWidth = docElement.clientWidth;
  const viewHeight = docElement.clientHeight;

  const viewArea = viewWidth * viewHeight;
  const viewMaxDim = Math.max(viewWidth, viewHeight);

  const impactFraction = impactRegionArea / viewArea;
  const distanceFraction = moveDistance / viewMaxDim;

  return impactFraction * distanceFraction;
};

// An list to record all the entries with startTime and score.
let watcher_entry_record = [];

// An object that tracks the document cumulative layout shift score.
// Usage:
//
//   const watcher = new ScoreWatcher;
//   ...
//   assert_equals(watcher.score, expectedScore);
//
// The score reflects only layout shifts that occur after the ScoreWatcher is
// constructed.
ScoreWatcher = function() {
  if (PerformanceObserver.supportedEntryTypes.indexOf("layout-shift") == -1)
    throw new Error("Layout Instability API not supported");
  this.score = 0;
  this.scoreWithInputExclusion = 0;
  const resetPromise = () => {
    this.promise = new Promise(resolve => {
      this.resolve = () => {
        resetPromise();
        resolve();
      }
    });
  };
  resetPromise();
  const observer = new PerformanceObserver(list => {
    list.getEntries().forEach(entry => {
      this.lastEntry = entry;
      this.score += entry.value;
      watcher_entry_record.push({startTime: entry.startTime, score: entry.value, hadRecentInput : entry.hadRecentInput});
      if (!entry.hadRecentInput)
        this.scoreWithInputExclusion += entry.value;
      this.resolve();
    });
  });
  observer.observe({entryTypes: ['layout-shift']});
};

ScoreWatcher.prototype.checkExpectation = function(expectation) {
  if (expectation.score != undefined)
    assert_equals(this.score, expectation.score);
  if (expectation.sources)
    check_sources(expectation.sources, this.lastEntry.sources);
};

ScoreWatcher.prototype.get_entry_record = function() {
  return watcher_entry_record;
};

check_sources = (expect_sources, actual_sources) => {
  assert_equals(expect_sources.length, actual_sources.length);
  let rect_match = (e, a) =>
      e[0] == a.x && e[1] == a.y && e[2] == a.width && e[3] == a.height;
  let match = e => a =>
      e.node === a.node &&
      rect_match(e.previousRect, a.previousRect) &&
      rect_match(e.currentRect, a.currentRect);
  for (let e of expect_sources)
    assert_true(actual_sources.some(match(e)), e.node + " not found");
};
