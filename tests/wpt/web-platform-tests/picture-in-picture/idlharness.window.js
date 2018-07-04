// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/picture-in-picture/

promise_test(async () => {
  const srcs = ['html', 'dom', 'picture-in-picture'];
  const [html, dom, pip] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(pip);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);

  idl_array.add_objects({
    Document: ['document'],
  });
  idl_array.test();
}, 'picture-in-picture interfaces.');
