// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/payment-handler/

promise_test(async () => {
  const idl = await fetch('/interfaces/payment-handler.idl').then(r => r.text());
  const idlArray = new IdlArray();
  idlArray.add_idls(idl);
  idlArray.add_untested_idls('interface ExtendableEvent {};');
  idlArray.add_untested_idls('dictionary ExtendableEventInit {};');
  idlArray.test();
  done();
}, 'Payment handler interfaces.');
