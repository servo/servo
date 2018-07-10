// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/payment-handler/

promise_test(async () => {
  const srcs = [
    'payment-handler',
    'service-workers',
    'dedicated-workers',
    'dom'
  ];
  const [idl, sw, dw, dom] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idlArray = new IdlArray();
  idlArray.add_idls(idl);
  idlArray.add_dependency_idls(sw);
  idlArray.add_dependency_idls(dw);
  idlArray.add_dependency_idls(dom);
  idlArray.test();
}, 'payment-handler interfaces.');
