// META: global=sharedworker
// META: timeout=long

// This is a regression test for https://crbug.com/844268, when a timeout of 10
// seconds was applied to XHR in Chrome. There should be no timeout unless the
// "timeout" parameter is set on the object.
test(t => {
  let xhr = new XMLHttpRequest();

  // For practical reasons, we can't wait forever. 12 seconds is long enough to
  // reliably reproduce the bug in Chrome.
  xhr.open('GET', 'resources/trickle.py?ms=1000&count=12', false);

  // The test will fail if this throws.
  xhr.send();
}, 'Sync XHR should not have a timeout');
