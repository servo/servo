// META: global=window,worker
// META: title=IDBDatabase.deleteObjectStore()
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>
// @author Odin Hørthe Omdal <mailto:odinho@opera.com>

'use_strict';

async_test(t => {
    let db;
    let add_success = false;

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;

        const objStore = db.createObjectStore("store", { autoIncrement: true });
        assert_equals(db.objectStoreNames[0], "store", "objectStoreNames");

        const rq_add = objStore.add(1);
        rq_add.onsuccess = function() { add_success = true; };
        rq_add.onerror = fail(t, 'rq_add.error');

        objStore.createIndex("idx", "a");
        db.deleteObjectStore("store");
        assert_equals(db.objectStoreNames.length, 0, "objectStoreNames.length after delete");
        assert_false(db.objectStoreNames.contains("store"));

        const exc = "InvalidStateError";
        assert_throws_dom(exc, function() { objStore.add(2); });
        assert_throws_dom(exc, function() { objStore.put(3); });
        assert_throws_dom(exc, function() { objStore.get(1); });
        assert_throws_dom(exc, function() { objStore.clear(); });
        assert_throws_dom(exc, function() { objStore.count(); });
        assert_throws_dom(exc, function() { objStore.delete(1); });
        assert_throws_dom(exc, function() { objStore.openCursor(); });
        assert_throws_dom(exc, function() { objStore.index("idx"); });
        assert_throws_dom(exc, function() { objStore.deleteIndex("idx"); });
        assert_throws_dom(exc, function() { objStore.createIndex("idx2", "a");
        });
    };

    open_rq.onsuccess = function() {
        assert_true(add_success, "First add was successful");
        t.done();
    }
}, 'Deleted object store\'s name should be removed from database\'s list. Attempting to use a \
deleted IDBObjectStore should throw an InvalidStateError');

async_test(t => {
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function(e) {
        const db = e.target.result;
        assert_throws_dom('NotFoundError', function() { db.deleteObjectStore('whatever'); });
        t.done();
    };
}, 'Attempting to remove an object store that does not exist should throw a NotFoundError');

async_test(t => {
    const keys = [];
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function(e) {
        const db = e.target.result;

        const objStore = db.createObjectStore("resurrected", { autoIncrement: true, keyPath: "k" });
        objStore.add({ k: 5 }).onsuccess = function(e) { keys.push(e.target.result); };
        objStore.add({}).onsuccess = function(e) { keys.push(e.target.result); };
        objStore.createIndex("idx", "i");
        assert_true(objStore.indexNames.contains("idx"));
        assert_equals(objStore.keyPath, "k", "keyPath");

        db.deleteObjectStore("resurrected");

        const objStore2 = db.createObjectStore("resurrected", { autoIncrement: true });
        objStore2.add("Unicorns'R'us").onsuccess = function(e) { keys.push(e.target.result); };
        assert_false(objStore2.indexNames.contains("idx"), "index exist on new objstore");
        assert_equals(objStore2.keyPath, null, "keyPath");

        assert_throws_dom("NotFoundError", function() { objStore2.index("idx"); });
    };

    open_rq.onsuccess = function(e) {
        assert_array_equals(keys, [5, 6, 1], "keys");
        t.done();
    };
}, 'Attempting to access an index that was deleted as part of object store deletion and then \
recreated using the same object store name should throw a NotFoundError');
