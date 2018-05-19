importScripts('/resources/testharness.js')

promise_test((test) => {
  return fetch('./sw.js').then((response) => {
    return new Promise((resolve, reject) => {
      step_timeout(() => {
        const entry = performance.getEntriesByName(response.url)[0]
        if (!entry) {
          reject('no entry: ' + response.url)
        }

        assert_not_equals(typeof entry.serverTiming,
          'undefined',
          'An instance of `PerformanceResourceTiming` should have a `serverTiming` attribute in the Service Worker context.')
        resolve()
      }, 250)
    })
  })
})
