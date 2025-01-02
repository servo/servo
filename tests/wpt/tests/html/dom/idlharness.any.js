// META: global=dedicatedworker,shadowrealm-in-window
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

"use strict";

idl_test(
  ["html"],
  ["wai-aria", "dom", "cssom", "touch-events", "uievents", "performance-timeline"],
  idlArray => {
    if (self.GLOBAL.isShadowRealm()) {
      return;
    }

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
