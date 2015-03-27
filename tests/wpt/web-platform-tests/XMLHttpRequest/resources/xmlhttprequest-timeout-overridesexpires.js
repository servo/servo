if (this.document === undefined)
  importScripts("xmlhttprequest-timeout.js");
/*
	Starts three requests:
	1) XHR to resource which will take a least 600ms with timeout initially set to 1000ms. After 800ms timeout is supposedly reset to 200ms, 
	   but the resource should have finished loading already. Asserts "load" fires.
	2) XHR with initial timeout set to 1000, after 400ms sets timeout to 300ms. Asserts "timeout" fires. 
	   (Originally new value was 200ms. Race condition-y. Setting the new timeout to 300ms would be a better test of the "measured from start of fetching" requirement.)
	3) XHR with initial timeout set to 200, after 400ms sets timeout to 500ms. Asserts "timeout" fires (the change happens when timeout already fired and the request is done).
*/
runTestRequests([ new RequestTracker(true, "timeout set to expiring value after load fires", TIME_NORMAL_LOAD, TIME_LATE_TIMEOUT, TIME_DELAY),
		  new RequestTracker(true, "timeout set to expired value before load fires", TIME_NORMAL_LOAD, TIME_REGULAR_TIMEOUT, TIME_DELAY+100),
		  new RequestTracker(true, "timeout set to non-expiring value after timeout fires", TIME_DELAY, TIME_REGULAR_TIMEOUT, 500) ]);
