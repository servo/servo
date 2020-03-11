// Number milliseconds to wait for CSS resources to load.
const numMillisecondsWait = 50;

// We use requestAnimationFrame() calls to force the user agent to paint and give enough
// time for FCP to show up in the performance timeline. Hence, set |numFramesWaiting| to
// 3 and use that constant whenever the test needs to wait for the next paint to occur.
const numFramesWaiting = 3;

function waitTime(t) {
  return new Promise(resolve => t.step_timeout(resolve, numMillisecondsWait));
}

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

// Asserts that there is currently no FCP reported, even after some wait.
function assertNoFirstContentfulPaint(t) {
  return waitTime(t).then(() => {
    return waitForAnimationFrames(numFramesWaiting);
  }).then(() => {
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

// Asserts that FCP is reported, possibly after some wait. The wait is needed
// because sometimes the FCP relies on some CSS resources to finish loading.
function assertFirstContentfulPaint(t) {
  return waitTime(t).then(() => {
    return waitForAnimationFrames(numFramesWaiting);
  }).then(() => {
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
