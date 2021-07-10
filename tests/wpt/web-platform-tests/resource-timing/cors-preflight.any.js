// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js

// Because apache decrements the Keep-Alive max value on each request, the
// transferSize will vary slightly between requests for the same resource.
const fuzzFactor = 3;  // bytes

const hostInfo = get_host_info();
const url = new URL('/resource-timing/resources/preflight.py', hostInfo['HTTP_REMOTE_ORIGIN']).href;

// The header bytes are expected to be > |minHeaderSize| and
// < |maxHeaderSize|. If they are outside this range the test will fail.
const minHeaderSize = 100;
const maxHeaderSize = 1024;

function checkResourceSizes() {
  const entries = performance.getEntriesByName(url);
  assert_equals(entries.length, 2, 'Wrong number of entries');
  assert_greater_than(entries[0].transferSize, 0, 'No-preflight transferSize');
  const lowerBound = entries[0].transferSize - fuzzFactor;
  const upperBound = entries[0].transferSize + fuzzFactor;
  assert_between_exclusive(entries[1].transferSize, lowerBound, upperBound, 'Preflighted transferSize');
}

promise_test(() => {
  const eatBody = response => response.arrayBuffer();
  const requirePreflight = {headers: {'X-Require-Preflight' : '1'}};
  return fetch(url)
      .then(eatBody)
      .then(() => fetch(url, requirePreflight))
      .then(eatBody)
      .then(checkResourceSizes);
}, 'PerformanceResourceTiming sizes fetch with preflight test');
