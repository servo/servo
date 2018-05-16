importScripts('/resources/testharness.js')

promise_test(async (test) => {
  return fetch('./sw.js').then(function(response) {
    assert_not_equals(typeof performance.getEntriesByName(response.url)[0].serverTiming,
      'undefined',
      'An instance of `PerformanceResourceTiming` should have a `serverTiming` attribute in the Service Worker context.')
    done()
  })
})
