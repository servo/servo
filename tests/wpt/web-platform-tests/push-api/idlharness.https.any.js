// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/push-api/

promise_test(async () => {
  const idl = await fetch('/interfaces/push-api.idl').then(r => r.text());
  const worker = await fetch('/interfaces/ServiceWorker.idl').then(r => r.text());
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(worker);
  idl_array.add_dependency_idls(dom);
  idl_array.test();
}, 'push-api interfaces');
