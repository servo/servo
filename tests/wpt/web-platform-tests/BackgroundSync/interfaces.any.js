// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/BackgroundSync/spec/

promise_test(async () => {
  const idl = await fetch('/interfaces/BackgroundSync.idl').then(r => r.text());
  const sw = await fetch('/interfaces/ServiceWorker.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());

  const idlArray = new IdlArray();
  idlArray.add_idls(idl);
  idlArray.add_dependency_idls(sw);
  idlArray.add_dependency_idls(html);
  idlArray.add_dependency_idls(dom);
  idlArray.test();
  done();
}, 'Background Sync interfaces.');
