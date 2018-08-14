// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/touch-events/

'use strict';

idl_test(
  ['touch-events'],
  ['uievents', 'dom', 'html'],
  idl_array => {
    idl_array.add_objects({
      Document: ['document'],
      GlobalEventHandlers: ['window', 'document', 'document.body'],
      Touch: ['new Touch({identifier: 1, target: document})'],
      TouchEvent: ['new TouchEvent("name")'],
    });
  }
);
