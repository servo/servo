// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/web-animations/#keyframeeffect

'use strict';

promise_test(async () => {
  const srcs = ['web-animations', 'html'];
  const [idl, html] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idlArray = new IdlArray();
  idlArray.add_idls(idl, {
    only: [
      'IterationCompositeOperation',
      'CompositeOperation',
      'KeyframeEffectOptions',
      'KeyframeEffect',
    ]
  });
  idlArray.add_untested_idls('interface CSSPseudoElement {};');
  idlArray.add_dependency_idls(idl);
  idlArray.add_dependency_idls(html);
  idlArray.add_objects({
    KeyframeEffect: ['new KeyframeEffect(null, null)'],
  });

  idlArray.test();
  done();
}, 'KeyframeEffect interface.');
