if (this.document === undefined)
  importScripts("xmlhttprequest-timeout.js");

runTestRequests([ new RequestTracker(true, "no time out scheduled, load fires normally", 0),
	          new RequestTracker(true, "load fires normally", TIME_NORMAL_LOAD),
	          new RequestTracker(true, "timeout hit before load", TIME_REGULAR_TIMEOUT) ]);
