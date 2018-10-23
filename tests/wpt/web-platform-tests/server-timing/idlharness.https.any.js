// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/server-timing/

promise_test(async () => {
  const idl = await fetch('/interfaces/server-timing.idl').then(r => r.text());
  const res = await fetch('/interfaces/resource-timing.idl').then(r => r.text());
  const perf = await fetch('/interfaces/performance-timeline.idl').then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(res);
  idl_array.add_dependency_idls(perf);
  idl_array.add_objects({
    Performance: ['performance'],
  });
  idl_array.test();
}, 'Test server-timing IDL implementation');
