'use strict';

importScripts('/resources/testharness.js');
importScripts('/resources/WebIDLParser.js', '/resources/idlharness.js');

promise_test(async function() {
  const idls = await fetch('/interfaces/background-fetch.idl').then(r => r.text());
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());

  var idlArray = new IdlArray();
  idlArray.add_untested_idls('interface ServiceWorkerRegistration {};');
  idlArray.add_untested_idls('[SecureContext, Exposed = (Window, Worker)] interface ServiceWorkerGlobalScope {};');
  idlArray.add_untested_idls('interface ExtendableEvent{};');
  idlArray.add_untested_idls('dictionary ExtendableEventInit{};');
  idlArray.add_untested_idls(dom, { only: ['EventTarget'] });
  idlArray.add_idls(idls);
  idlArray.test();
}, 'Exposed interfaces in a Service Worker.');

done();
