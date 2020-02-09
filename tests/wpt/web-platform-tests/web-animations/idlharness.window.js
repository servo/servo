// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['web-animations'],
  ['dom', 'html'],
  idl_array => {
    idl_array.add_objects({
      Animation: ['new Animation()'],
      AnimationPlaybackEvent: ['new AnimationPlaybackEvent("cancel")'],
      Document: ['document'],
      DocumentTimeline: ['document.timeline'],
      KeyframeEffect: ['new KeyframeEffect(null, null)'],
      ShadowRoot: ['shadowRoot'],
    });
    self.shadowRoot = document.createElement("div").attachShadow({mode: "open"});
  }
);
