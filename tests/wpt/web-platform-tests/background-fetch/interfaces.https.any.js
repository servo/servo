// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/background-fetch/

promise_test(async () => {
  const srcs = [
    'background-fetch',
    'dedicated-workers',
    'service-workers',
    'dom'
  ];
  const [idls, worker, serviceWorker, dom] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  var idlArray = new IdlArray();
  idlArray.add_idls(idls);
  idlArray.add_dependency_idls(serviceWorker);
  idlArray.add_dependency_idls(worker);
  idlArray.add_dependency_idls(dom);
  idlArray.test();
}, 'background-fetch interfaces');
