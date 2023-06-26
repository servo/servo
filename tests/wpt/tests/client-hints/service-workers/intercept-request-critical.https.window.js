// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/util.js
promise_test((t) =>
  ch_sw_test(t, 'critical-ch/intercept-request.js', 'critical-ch/foo.html', 'FAIL'),
  "Service workers succsefully receives hints from request");
