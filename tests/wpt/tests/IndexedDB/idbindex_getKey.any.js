// META: global=window,worker
// META: title=IDBIndex.getKey()
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>
// @author Intel <http://www.intel.com>

'use_strict';

async_test(t => {
    let db;
    const record = { key: 1, indexedProperty: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const objStore = db.createObjectStore("test", { keyPath: "key" });
      objStore.createIndex("index", "indexedProperty");

      objStore.add(record);
    };

    open_rq.onsuccess = function(e) {
      let rq = db.transaction("test", "readonly")
        .objectStore("test");

      rq = rq.index("index");

      rq = rq.getKey("data");

      rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result, record.key);
        t.done();
      });
    };
}, 'getKey() returns the record\'s primary key');

async_test(t => {
    let db;
    const records = [
      { key: 1, indexedProperty: "data" },
      { key: 2, indexedProperty: "data" },
      { key: 3, indexedProperty: "data" }
    ];

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      var objStore = db.createObjectStore("test", { keyPath: "key" });
      objStore.createIndex("index", "indexedProperty");

      for (let i = 0; i < records.length; i++)
        objStore.add(records[i]);
    };

    open_rq.onsuccess = function(e) {
      const rq = db.transaction("test", "readonly")
        .objectStore("test")
        .index("index")
        .getKey("data");

      rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result, records[0].key);
        t.done();
      });
    };
}, 'getKey() returns the record\'s primary key where the index contains duplicate values');

async_test(t => {
    let db;
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const rq = db.createObjectStore("test", { keyPath: "key" })
                  .createIndex("index", "indexedProperty")
                  .getKey(1);

      rq.onsuccess = t.step_func(function(e) {
          assert_equals(e.target.result, undefined);
          t.done();
      });
    };
}, 'getKey() attempt to retrieve the primary key of a record that doesn\'t exist');

async_test(t => {
    let db;

    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const store = db.createObjectStore("store", { keyPath: "key" });
      store.createIndex("index", "indexedProperty");

      for (let i = 0; i < 10; i++) {
        store.add({ key: i, indexedProperty: "data" + i });
      }
    };

    open_rq.onsuccess = function(e) {
      const rq = db.transaction("store", "readonly")
        .objectStore("store")
        .index("index")
        .getKey(IDBKeyRange.bound('data4', 'data7'));

      rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result, 4);

        step_timeout(function () { t.done(); }, 4)
      });
    };
}, 'getKey() returns the key of the first record within the range');

async_test(t => {
    let db;
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;

      const index = db.createObjectStore("test", { keyPath: "key" })
        .createIndex("index", "indexedProperty");

      assert_throws_dom("DataError", function () {
        index.getKey(NaN);
      });
      t.done();
    };
}, 'getKey() throws DataError when using invalid key');

async_test(t => {
    let db;
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const store = db.createObjectStore("store", { keyPath: "key" });
      const index = store.createIndex("index", "indexedProperty");

      store.add({ key: 1, indexedProperty: "data" });
      store.deleteIndex("index");

      assert_throws_dom("InvalidStateError", function () {
        index.getKey("data");
      });
      t.done();
    };
}, 'getKey() throws InvalidStateError when the index is deleted');

async_test(t => {
    let db;

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const store = db.createObjectStore("store", { keyPath: "key" });
      const index = store.createIndex("index", "indexedProperty");
      store.add({ key: 1, indexedProperty: "data" });
    };

    open_rq.onsuccess = function(e) {
      db = e.target.result;
      const tx = db.transaction('store', 'readonly');
      const index = tx.objectStore('store').index('index');
      tx.abort();

      assert_throws_dom("TransactionInactiveError", function () {
        index.getKey("data");
      });
      t.done();
    };
}, 'getKey() throws TransactionInactiveError on aborted transaction');

async_test(t => {
    let db;

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        const store = db.createObjectStore("store", { keyPath: "key" });
        const index = store.createIndex("index", "indexedProperty");
        store.add({ key: 1, indexedProperty: "data" });

        e.target.transaction.abort();

        assert_throws_dom("InvalidStateError", function () {
        index.getKey("data");
        });
        t.done();
    };
}, 'getKey() throws InvalidStateError on index deleted by aborted upgrade');
