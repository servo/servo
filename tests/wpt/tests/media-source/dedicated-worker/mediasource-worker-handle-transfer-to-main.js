importScripts('mediasource-message-util.js');

// Note, we do not use testharness.js utilities within the worker context
// because it also communicates using postMessage to the main HTML document's
// harness, and would confuse the test case message parsing there.

// Just obtain a MediaSourceHandle and transfer it to creator of our context.
let handle = new MediaSource().handle;
postMessage(
    {subject: messageSubject.HANDLE, info: handle}, {transfer: [handle]});
