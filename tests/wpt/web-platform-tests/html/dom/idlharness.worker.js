"use strict";

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

idl_test(
  ["html"],
  ["wai-aria", "dom", "cssom", "touch-events", "uievents"],
  idlArray => {
    idlArray.add_untested_idls('typedef Window WindowProxy;');
    idlArray.add_objects({
      WorkerLocation: ['self.location'],
      WorkerNavigator: ['self.navigator'],
      EventSource: ['new EventSource("http://invalid")'],
      Worker: [],
      MessageEvent: ['new MessageEvent("message", { data: 5 })'],
      DedicatedWorkerGlobalScope: ['self'],
    });
  }
);

done();
