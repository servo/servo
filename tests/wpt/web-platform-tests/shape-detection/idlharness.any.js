// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// See: https://wicg.github.io/shape-detection-api/

promise_test(async () => {
  const idl = await fetch('/interfaces/shape-detection-api.idl').then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.test();
}, 'Test shape-detection IDL interface');
