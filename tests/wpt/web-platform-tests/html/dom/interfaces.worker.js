"use strict";

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

idl_test(
  ["html"],
  ["dom", "cssom", "touch-events", "uievents"],
  idlArray => {
    idlArray.add_objects({
      WorkerNavigator: ['self.navigator'],
      WebSocket: ['new WebSocket("ws://foo")'],
      CloseEvent: ['new CloseEvent("close")'],
      Worker: [],
      MessageEvent: ['new MessageEvent("message", { data: 5 })'],
      DedicatedWorkerGlobalScope: ['self'],
    });
  }
);

done();
