// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/manifest/

'use strict';

promise_test(async () => {
  const srcs = ['appmanifest', 'dom', 'html'];
  const [idl, dom, html] = await Promise.all(
    srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(dom);
  idl_array.add_objects({
    Window: ['window'],
    BeforeInstallPromptEvent: ['new BeforeInstallPromptEvent("type")'],
  });
  idl_array.test();
}, 'appmanifest interfaces');
