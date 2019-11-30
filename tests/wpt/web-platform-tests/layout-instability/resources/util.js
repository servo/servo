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
      this.score += entry.value;
      if (!entry.hadRecentInput)
        this.scoreWithInputExclusion += entry.value;
      this.resolve();
    });
  });
  observer.observe({entryTypes: ['layout-shift']});
};
