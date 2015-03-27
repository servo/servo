if (this.document === undefined)
  importScripts("xmlhttprequest-timeout.js");
/*
Sets up three requests to a resource that will take 0.6 seconds to load:
1) timeout first set to 1000ms, after 400ms timeout is set to 0, asserts load fires
2) timeout first set to 1000ms, after 200ms timeout is set to 400, asserts load fires (race condition..?!?)
3) timeout first set to 0, after 400ms it is set to 1000, asserts load fires
*/
runTestRequests([ new RequestTracker(true, "timeout disabled after initially set", TIME_NORMAL_LOAD, TIME_REGULAR_TIMEOUT, 0),
		  new RequestTracker(true, "timeout overrides load after a delay", TIME_NORMAL_LOAD, TIME_DELAY, TIME_REGULAR_TIMEOUT),
		  new RequestTracker(true, "timeout enabled after initially disabled", 0, TIME_REGULAR_TIMEOUT, TIME_NORMAL_LOAD * 10) ]);
