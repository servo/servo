// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/DOM-Parsing/

promise_test(async () => {
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());
  const idl = await fetch('/interfaces/DOM-Parsing.idl').then(r => r.text());
  const idlArray = new IdlArray();
  idlArray.add_untested_idls(dom);
  idlArray.add_idls(idl);
  idlArray.test();
  done();
}, 'Test driver');
