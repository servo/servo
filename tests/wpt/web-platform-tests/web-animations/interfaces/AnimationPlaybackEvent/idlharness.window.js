// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/web-animations/#animationplaybackevent

'use strict';

promise_test(async () => {
  const srcs = ['web-animations', 'dom'];
  const [idl, dom] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idlArray = new IdlArray();
  idlArray.add_idls(idl, {
    only: [
      'AnimationPlaybackEventInit',
      'AnimationPlaybackEvent',
    ]
  });
  idlArray.add_dependency_idls(dom);
  idlArray.add_objects({
    AnimationPlaybackEvent: ['new AnimationPlaybackEvent("cancel")'],
  });

  idlArray.test();
}, 'AnimationPlaybackEvent interface.');
