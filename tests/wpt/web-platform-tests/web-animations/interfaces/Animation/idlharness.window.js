// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/web-animations/#animation

'use strict';

promise_test(async () => {
  const srcs = ['web-animations', 'dom', 'html'];
  const [idl, dom, html] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idlArray = new IdlArray();
  idlArray.add_idls(idl, {only: ['Animation', 'AnimationPlayState']});
  idlArray.add_dependency_idls(idl);
  idlArray.add_dependency_idls(dom);
  idlArray.add_dependency_idls(html);
  idlArray.add_untested_idls('interface CSSPseudoElement {};');
  idlArray.add_objects( { Animation: ['new Animation()'] } );
  idlArray.test();
}, 'Animation interface.');
