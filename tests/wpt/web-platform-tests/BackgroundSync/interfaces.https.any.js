// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/BackgroundSync/spec/

promise_test(async () => {
  const srcs = [
    'BackgroundSync',
    'service-workers',
    'html',
    'dom'
  ];
  const [idl, sw, html, dom] = await Promise.all(
    srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idlArray = new IdlArray();
  idlArray.add_idls(idl);
  idlArray.add_dependency_idls(sw);
  idlArray.add_dependency_idls(html);
  idlArray.add_dependency_idls(dom);
  idlArray.test();
  done();
}, 'Background Sync interfaces.');
