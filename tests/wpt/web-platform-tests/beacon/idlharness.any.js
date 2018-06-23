// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/beacon/

promise_test(async () => {
  const idl = await fetch('/interfaces/beacon.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_objects({
    Navigator: ['navigator'],
  });
  idl_array.test();
}, 'beacon interfaces');
