// META: title=IDBDatabase.transaction()
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
    let db;
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function () { };
    open_rq.onsuccess = function (e) {
        db = e.target.result;

        assert_throws_dom('NotFoundError', function () { db.transaction('non-existing'); });
        t.done();
    };
}, "Attempt to open a transaction with invalid scope");

async_test(t => {
    let db;
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function (e) {
        db = e.target.result;
        db.createObjectStore('readonly');
    };
    open_rq.onsuccess = function (e) {
        var txn = db.transaction('readonly', 'readonly');
        assert_equals(txn.mode, "readonly", 'txn.mode');

        t.done();
    };
}, "Opening a transaction defaults to a read-only mode");

async_test(t => {
    let db;
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function (e) {
        db = e.target.result;
        db.createObjectStore('test');
    };

    open_rq.onsuccess = function (e) {
        db.close();

        assert_throws_dom('InvalidStateError',
            function () { db.transaction('test', 'readonly'); });

        t.done();
    };
}, "Attempt to open a transaction from closed database connection");

async_test(t => {
    let db;
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function (e) {
        db = e.target.result;
        db.createObjectStore('test');
    };

    open_rq.onsuccess = function (e) {
        assert_throws_js(TypeError,
            function () { db.transaction('test', 'whatever'); });

        t.done();
    };
}, "Attempt to open a transaction with invalid mode");

async_test(t => {
    let db;
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function () { };
    open_rq.onsuccess = function (e) {
        db = e.target.result;
        assert_throws_dom('InvalidAccessError', function () { db.transaction([]); });
        t.done();
    };
}, "If storeNames is an empty list, the implementation must throw a DOMException of type InvalidAccessError");