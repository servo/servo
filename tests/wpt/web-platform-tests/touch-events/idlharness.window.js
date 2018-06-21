// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/touch-events/

'use strict';

promise_test(async () => {
  const srcs = ['touch-events', 'uievents', 'dom', 'html'];
  const [idl, uievents, dom, html] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(uievents);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);
  idl_array.add_objects({
    Document: ['document'],
    GlobalEventHandlers: ['window', 'document', 'document.body'],
    Touch: ['new Touch({identifier: 1, target: document})'],
    TouchEvent: ['new TouchEvent("name")'],
  });
  idl_array.test();
}, 'Test IDL implementation of touch-events API');
