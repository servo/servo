// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://console.spec.whatwg.org/

promise_test(async () => {
  const srcs = ['console'];
  const [idl] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.test();
}, 'console interfaces');
