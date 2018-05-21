// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

promise_test(async() => {
  const text = await (await fetch('/interfaces/encoding.idl')).text();
  const idl_array = new IdlArray();
  idl_array.add_idls(text);
  idl_array.add_objects({
    TextEncoder: ['new TextEncoder()'],
    TextDecoder: ['new TextDecoder()']
  });
  idl_array.test();
}, 'Encoding Standard IDL');
