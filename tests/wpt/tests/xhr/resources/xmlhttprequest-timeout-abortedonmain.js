/*
This test sets up two requests:
one that gets abort()ed from a 0ms timeout (0ms will obviously be clamped to whatever the implementation's minimal value is), asserts abort event fires
one that will be aborted after TIME_DELAY, (with a timeout at TIME_REGULAR_TIMEOUT) asserts abort event fires. Does not assert that the timeout event does *not* fire.
*/

runTestRequests([ ["AbortedRequest", true, "abort() from a 0ms timeout", 0],
                  ["AbortedRequest", true, "aborted after TIME_DELAY", TIME_DELAY] ]);
