//META: script=/service-workers/service-worker/resources/test-helpers.sub.js
//META: script=resources/util.js
promise_test((t) =>
  ch_sw_test(t, 'critical-ch/new-request.js', 'critical-ch/foo.html', 'FAIL'),
  "Service worker does NOT generate client hints in a new request");
