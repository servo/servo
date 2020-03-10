// We use requestAnimationFrame() calls to force the user agent to paint. Hence, set
// |numFramesWaiting| to 3 and use that constant whenever the test needs to wait for
// the next paint to occur.
const numFramesWaiting = 3;

function waitForAnimationFrames(count) {
  return new Promise(resolve => {
    if (count-- <= 0) {
      resolve();
    } else {
      requestAnimationFrame(() => {
        waitForAnimationFrames(count).then(resolve);
      });
    }
  });
}

function assertNoFirstContentfulPaint() {
  return waitForAnimationFrames(numFramesWaiting).then(() => {
    return new Promise((resolve, reject) => {
      const observer = new PerformanceObserver(entryList =>{
        const entries = entryList.getEntriesByName('first-contentful-paint');
        observer.disconnect();
        if (entries.length > 0)
          reject('Received a first contentful paint entry.');
        else
          resolve();
      });
      observer.observe({type: 'paint', buffered: true});
      observer.observe({type: 'mark'});
      performance.mark('flush');
    });
  });
}

function assertFirstContentfulPaint() {
  return waitForAnimationFrames(numFramesWaiting).then(() => {
    return new Promise((resolve, reject) => {
      const observer = new PerformanceObserver(entryList =>{
        const entries = entryList.getEntriesByName('first-contentful-paint');
        observer.disconnect();
        if (entries.length === 0)
          reject('Did not receive a first contentful paint entry.');
        else {
          resolve();
        }
      });
      observer.observe({type: 'paint', buffered: true});
      observer.observe({type: 'mark'});
      performance.mark('flush');
    });
  });
}
