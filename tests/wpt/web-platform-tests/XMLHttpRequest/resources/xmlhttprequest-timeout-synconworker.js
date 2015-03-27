if (this.document === undefined){
  importScripts("xmlhttprequest-timeout.js");
}else{
	throw "This test expects to be run as a Worker";
}

/* NOT TESTED: setting timeout before calling open( ... , false) in a worker context. The test code always calls open() first. */

runTestRequests([ new RequestTracker(false, "no time out scheduled, load fires normally", 0),
		  new RequestTracker(false, "load fires normally", TIME_NORMAL_LOAD),
		  new RequestTracker(false, "timeout hit before load", TIME_REGULAR_TIMEOUT) ]);
