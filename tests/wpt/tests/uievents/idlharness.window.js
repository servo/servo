// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

idl_test(
  ['uievents'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      FocusEvent: ['new FocusEvent("event")'],
      MouseEvent: ['new MouseEvent("event")'],
      WheelEvent: ['new WheelEvent("event")'],
      KeyboardEvent: ['new KeyboardEvent("event")'],
      CompositionEvent: ['new CompositionEvent("event")'],
      UIEvent: ['new UIEvent("event")'],
      InputEvent: ['new InputEvent("event")'],
      TextEvent: ['(() => { const ev = document.createEvent("TextEvent"); ev.initTextEvent("event"); return ev; })()'],
    });
  }
);
