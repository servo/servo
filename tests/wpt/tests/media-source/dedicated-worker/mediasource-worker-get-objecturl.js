importScripts("mediasource-worker-util.js");

// Note, we do not use testharness.js utilities within the worker context
// because it also communicates using postMessage to the main HTML document's
// harness, and would confuse the test case message parsing there.

onmessage = function(evt) {
  postMessage({ subject: messageSubject.ERROR, info: "No message expected by Worker"});
};

let util = new MediaSourceWorkerUtil();

postMessage({ subject: messageSubject.OBJECT_URL, info: URL.createObjectURL(util.mediaSource) });
