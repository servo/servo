// META: global=window,worker
// META: title=IDBObjectStore.openCursor() - invalid
// META: script=resources/support.js

'use strict';

indexeddb_test(
    function(t, db, tx) {
      let objStore = db.createObjectStore('test');
      objStore.createIndex('index', '');

      objStore.add('data', 1);
      objStore.add('data2', 2);
    },
    function(t, db, tx) {
      let idx =
          db.transaction('test', 'readonly').objectStore('test').index('index');

      assert_throws_dom('DataError', function() {
        idx.openCursor({lower: 'a'});
      });

      assert_throws_dom('DataError', function() {
        idx.openCursor({lower: 'a', lowerOpen: false});
      });

      assert_throws_dom('DataError', function() {
        idx.openCursor(
            {lower: 'a', lowerOpen: false, upper: null, upperOpen: false});
      });

      t.done();
    },
    'IDBObjectStore.openCursor() - invalid - pass something other than number');
