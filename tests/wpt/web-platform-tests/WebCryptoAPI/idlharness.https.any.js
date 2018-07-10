// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webcrypto/Overview.html

promise_test(async () => {
  const idl = await fetch(`/interfaces/WebCryptoAPI.idl`).then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_objects({
    Crypto: ['crypto'],
    SubtleCrypto: ['crypto.subtle']
  });
  idl_array.test();
}, 'WebCryptoAPI interfaces');
