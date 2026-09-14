// META: title=Deleting a cookie via a past expiration date via document.cookie

"use strict";
setup({ single_test: true });

// Regression test for https://github.com/jsdom/jsdom/issues/3781

document.cookie = "test=1";
assert_equals(document.cookie, "test=1");

document.cookie = "test=1; expires=Thu, 01 Jan 1970 00:00:00 GMT";
assert_equals(document.cookie, "");

done();
