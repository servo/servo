if (this.document === undefined)
  importScripts("xmlhttprequest-timeout.js");

runTestRequests([ ["RequestTracker", true, "no time out scheduled, load fires normally", 0],
                  ["RequestTracker", true, "load fires normally", TIME_NORMAL_LOAD],
                  ["RequestTracker", true, "timeout hit before load", TIME_REGULAR_TIMEOUT] ]);
