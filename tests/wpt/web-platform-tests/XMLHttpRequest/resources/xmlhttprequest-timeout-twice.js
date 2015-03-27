if (this.document === undefined)
  importScripts("xmlhttprequest-timeout.js");

runTestRequests([ new RequestTracker(true, "load fires normally with no timeout set, twice", 0, TIME_REGULAR_TIMEOUT, 0),
		  new RequestTracker(true, "load fires normally with same timeout set twice", TIME_NORMAL_LOAD, TIME_REGULAR_TIMEOUT, TIME_NORMAL_LOAD),
		  new RequestTracker(true, "timeout fires normally with same timeout set twice", TIME_REGULAR_TIMEOUT, TIME_DELAY, TIME_REGULAR_TIMEOUT) ]);
