// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/web-animations/#documenttimeline

'use strict';

promise_test(async () => {
  const text = await fetch('/interfaces/web-animations.idl').then(r => r.text());
  const idlArray = new IdlArray();
  idlArray.add_idls(text, {
    only: [
      'AnimationTimeline',
      'DocumentTimelineOptions',
      'DocumentTimeline',
    ]
  });
  idlArray.add_objects({ DocumentTimeline: ['document.timeline'] });

  idlArray.test();
  done();
}, 'DocumentTimeline interface.');
