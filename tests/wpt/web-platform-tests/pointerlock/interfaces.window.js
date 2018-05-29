// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/pointerlock/

promise_test(async () => {
  const html = await fetch('/interfaces/html.idl').then(r => r.text());
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());
  const uievents = await fetch('/interfaces/uievents.idl').then(r => r.text());
  const idl = await fetch('/interfaces/pointerlock.idl').then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(uievents);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);

  idl_array.add_objects({
    Document: ["window.document"],
    Element: ["window.document.documentElement"],
    MouseEvent: ["new MouseEvent('foo')"]
  });
  idl_array.test();
}, 'pointerlock interfaces.');
