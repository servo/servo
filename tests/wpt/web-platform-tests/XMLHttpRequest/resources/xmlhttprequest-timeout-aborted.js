if (this.document === undefined)
  importScripts("xmlhttprequest-timeout.js");
/*
This sets up three requests:
The first request will only be open()ed, not aborted, timeout will be 400 but will never triggered because send() isn't called. 
After a 1 second delay, the test asserts that no load/error/timeout/abort events fired

Second request will be aborted immediately after send(), test asserts that abort fired

Third request is set up to call abort() after a 1 second delay, but it also has a 400ms timeout. Asserts that timeout fired.
(abort() is called 600ms later and should not fire an abort event per spec. This is untested!)
*/
runTestRequests([ new AbortedRequest(false),
		  new AbortedRequest(true, -1),
		  new AbortedRequest(true, TIME_NORMAL_LOAD) ]);
