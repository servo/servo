// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://webbluetoothcg.github.io/web-bluetooth/

promise_test(async () => {
  const srcs = ['web-bluetooth', 'dom', 'html', 'permissions'];
  const [idl, dom, html, permissions] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(permissions);
  idl_array.test();
}, 'web-bluetooth interfaces.');
