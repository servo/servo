// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://compat.spec.whatwg.org/

promise_test(async () => {
  const idl = await fetch('/interfaces/compat.idl').then(r => r.text());
  const idlArray = new IdlArray();
  idlArray.add_untested_idls('interface Window {};');
  idlArray.add_untested_idls('interface EventTarget{};');
  idlArray.add_untested_idls('interface HTMLBodyElement{};');
  idlArray.add_idls(idl);
  idlArray.test();
  done();
}, 'compat interfaces.');
