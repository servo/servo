'use strict';

importScripts('/resources/testharness.js');
importScripts('/resources/WebIDLParser.js', '/resources/idlharness.js');

promise_test(t => {
  return fetch('interfaces.idl')
    .then(response => response.text())
    .then(idls => {
      var idl_array = new IdlArray();

      idl_array.add_untested_idls('interface Navigator {};');
      idl_array.add_untested_idls('[Exposed=Worker] interface WorkerNavigator {};');

      idl_array.add_idls(idls);

      idl_array.add_objects({
        StorageManager: ['navigator.storage']
      });

      idl_array.test();
      t.done();
    });
}, 'Storage API IDL test');

done();
