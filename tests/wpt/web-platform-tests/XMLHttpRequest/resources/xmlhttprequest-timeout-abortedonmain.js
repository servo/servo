/*
This test sets up two requests: 
one that gets abort()ed from a 0ms timeout (0ms will obviously be clamped to whatever the implementation's minimal value is), asserts abort event fires
one that will be aborted after 200ms (TIME_DELAY), (with a timeout at 400ms) asserts abort event fires. Does not assert that the timeout event does *not* fire.
*/

runTestRequests([ new AbortedRequest(true, 0),
		  new AbortedRequest(true, TIME_DELAY) ]);
