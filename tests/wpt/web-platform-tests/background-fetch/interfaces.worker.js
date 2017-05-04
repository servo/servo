'use strict';

importScripts('/resources/testharness.js');
importScripts('/resources/WebIDLParser.js', '/resources/idlharness.js');

promise_test(function() {
  return fetch('interfaces.idl')
    .then(response => response.text())
    .then(idls => {
      var idlArray = new IdlArray();
      idlArray.add_untested_idls('interface ServiceWorkerRegistration {};');
      idlArray.add_untested_idls('[Exposed=ServiceWorker] interface ServiceWorkerGlobalScope {};');
      idlArray.add_idls(idls);
      idlArray.test();
    });
}, 'Exposed interfaces in a Service Worker.');
