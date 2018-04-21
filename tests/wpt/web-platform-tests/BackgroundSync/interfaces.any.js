// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/BackgroundSync/spec/

promise_test(async () => {
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());
  const sw = await fetch('/interfaces/ServiceWorker.idl').then(r => r.text());
  const idl = await fetch('/interfaces/BackgroundSync.idl').then(response => response.text());

  const idlArray = new IdlArray();
  idlArray.add_untested_idls(dom, { only: ['Event', 'EventInit', 'EventTarget'] });
  idlArray.add_untested_idls(html, { only: [
    'WorkerGlobalScope',
    'WindowOrWorkerGlobalScope'
  ] });
  idlArray.add_untested_idls(sw, { only: [
    'ServiceWorkerRegistration',
    'ServiceWorkerGlobalScope',
    'ExtendableEvent',
    'ExtendableEventInit',
  ] });
  idlArray.add_idls(idl);
  idlArray.test();
  done();
}, 'Background Sync interfaces.');
