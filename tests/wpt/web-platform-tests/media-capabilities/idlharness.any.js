// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://wicg.github.io/media-capabilities/

'use strict';

promise_test(async () => {
  const idl = await fetch('/interfaces/media-capabilities.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());
  const cssomView = await fetch('/interfaces/cssom-view.idl').then(r => r.text());

  var idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(cssomView);

  idl_array.add_objects({
    Navigator: ['navigator']
  });

  idl_array.test();
}, 'Test IDL implementation of Media Capabilities');
