// This worker attempts to call respondWith() asynchronously after the
// fetch event handler finished. It reports back to the test whether
// an exception was thrown.

// These get reset at the start of a test case.
let reportResult;
let resultPromise;

// The test page sends a message to tell us that a new test case is starting.
// We expect a fetch event after this.
self.addEventListener('message', (event) => {
  resultPromise = new Promise((resolve) => {
    reportResult = resolve;
  });

  // Keep the worker alive until the test case finishes, and report
  // back the result to the test page.
  event.waitUntil(resultPromise.then(result => {
    event.source.postMessage(result);
  }));
});

// Calls respondWith() and reports back whether an exception occurred.
function tryRespondWith(event) {
  try {
    event.respondWith(new Response());
    reportResult({didThrow: false});
  } catch (error) {
    reportResult({didThrow: true, error: error.name});
  }
}

function respondWithInTask(event) {
  setTimeout(() => {
    tryRespondWith(event);
  }, 0);
}

function respondWithInMicrotask(event) {
  Promise.resolve().then(() => {
    tryRespondWith(event);
  });
}

self.addEventListener('fetch', function(event) {
  const path = new URL(event.request.url).pathname;
  const test = path.substring(path.lastIndexOf('/') + 1);

  // If this is a test case, try respondWith() and report back to the test page
  // the result.
  if (test == 'respondWith-in-task') {
    respondWithInTask(event);
  } else if (test == 'respondWith-in-microtask') {
    respondWithInMicrotask(event);
  }
});
