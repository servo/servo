// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webappsec-csp/embedded/

'use strict';

promise_test(async () => {
  const idl = await fetch('/interfaces/csp-embedded-enforcement.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(dom);
  idl_array.add_objects({
    HTMLIFrameElement: ['document.createElement("iframe")'],
  });
  idl_array.test();
}, 'csp-embedded-enforcement IDL');
