// META: global=window,worker
// META: title=IDBIndex.count()
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>
// @author Odin Hørthe Omdal <mailto:odinho@opera.com>
// @author Intel <http://www.intel.com>

'use_strict';

async_test(t => {
    let db;

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const store = db.createObjectStore("store", { autoIncrement: true });
      store.createIndex("index", "indexedProperty");
      for (let i = 0; i < 10; i++) {
        store.add({ indexedProperty: "data" + i });
      }
    };

    open_rq.onsuccess = function(e) {
      const rq = db.transaction("store", "readonly", { durability: 'relaxed' })
        .objectStore("store")
        .index("index")
        .count();

      rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result, 10);
        t.done();
      });
    };
}, 'count() returns the number of records in the index');

async_test(t => {
    let db;

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const store = db.createObjectStore("store", { autoIncrement: true });
      store.createIndex("index", "indexedProperty");

      for (let i = 0; i < 10; i++) {
        store.add({ indexedProperty: "data" + i });
      }
    };

    open_rq.onsuccess = function(e) {
      const rq = db.transaction("store", "readonly", { durability: 'relaxed' })
        .objectStore("store")
        .index("index")
        .count(IDBKeyRange.bound('data0', 'data4'));

      rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result, 5);
        t.done();
      });
    };
}, 'count() returns the number of records that have keys within the range');

async_test(t => {
    let db;

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;

      const store = db.createObjectStore("store", { autoIncrement: true });
      store.createIndex("myindex", "idx");

      for (let i = 0; i < 10; i++)
        store.add({ idx: "data_" + (i%2) });

      store.index("myindex").count("data_0").onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result, 5, "count(data_0)");
        t.done();
      });
    };
}, 'count() returns the number of records that have keys with the key');

async_test(t => {
    let db;

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const store = db.createObjectStore("store", { autoIncrement: true });
      store.createIndex("index", "indexedProperty");

      for (let i = 0; i < 10; i++) {
        store.add({ indexedProperty: "data" + i });
      }
    };

    open_rq.onsuccess = function(e) {
      const index = db.transaction("store", "readonly", { durability: 'relaxed' })
        .objectStore("store")
        .index("index");

      assert_throws_dom("DataError", function () {
        index.count(NaN);
      });

      t.done();
    };
}, 'count() throws DataError when using invalid key');
