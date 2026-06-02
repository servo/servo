// META: global=window,worker
// META: title=IDBObjectStore.get()
// META: script=resources/support.js

"use strict";

function createDbRecordAndValidate(record, t) {
  const openRequest = createdb(t);

  openRequest.onupgradeneeded = t.step_func(event => {
    const db = event.target.result;
    const store = db.createObjectStore('store', {keyPath: 'key'});
    store.add(record);

    openRequest.onsuccess = t.step_func(event => {
      const rq = db.transaction('store', 'readonly')
                     .objectStore('store')
                     .get(record.key);

      rq.onsuccess = t.step_func(event => {
        const result = event.target.result;
        assert_equals(result.key.valueOf(), result.key.valueOf());
        assert_equals(result.property, record.property);
        t.done();
      });
    });
  });
}

async_test(t => {
  const record = {key: 3.14159265, property: 'data'};
  createDbRecordAndValidate(record, t);
}, 'Key is a number');

async_test(t => {
  const record = {key: 'this is a key that\'s a string', property: 'data'};
  createDbRecordAndValidate(record, t);
}, 'Key is a string');

async_test(t => {
  const record = {key: new Date(), property: 'data'};
  createDbRecordAndValidate(record, t);
}, 'Key is a date');

async_test(t => {
  const open_rq = createdb(t);

  open_rq.onupgradeneeded = t.step_func(event => {
    const db = event.target.result;
    const rq = db.createObjectStore('store', {keyPath: 'key'}).get(1);

    rq.onsuccess = t.step_func(event => {
      assert_equals(event.target.result, undefined);
      t.done();
    });
  });
}, 'Attempts to retrieve a record that doesn\'t exist');

async_test(t => {
  let db;
  const open_rq = createdb(t);

  open_rq.onupgradeneeded = t.step_func(event => {
    db = event.target.result;
    const os = db.createObjectStore('store');

    for (let i = 0; i < 10; i++) {
      os.add(`data${i}`, i);
    }
  });

  open_rq.onsuccess = t.step_func(event => {
    const rq = db.transaction('store', 'readonly')
                   .objectStore('store')
                   .get(IDBKeyRange.bound(3, 6));

    rq.onsuccess = t.step_func(event => {
      assert_equals(event.target.result, 'data3', 'get(3-6)');
      t.done();
    });
  });
}, 'Returns the record with the first key in the range');

async_test(t => {
  let db;
  const open_rq = createdb(t);

  open_rq.onupgradeneeded = t.step_func(event => {
    db = event.target.result;
    db.createObjectStore('store', {keyPath: 'key'});
  });

  open_rq.onsuccess = t.step_func(event => {
    const store = db.transaction('store', 'readonly').objectStore('store');

    // Abort the transaction immediately.
    store.transaction.abort();

    // Accessing the store after the transaction aborts must throw
    // TransactionInactiveError.
    assert_throws_dom('TransactionInactiveError', () => {
      store.get(1);
    });

    t.done();
  });
}, 'When a transaction is aborted, throw TransactionInactiveError');

async_test(t => {
  let db;
  const open_rq = createdb(t);

  open_rq.onupgradeneeded = t.step_func(event => {
    db = event.target.result;
    db.createObjectStore('store', {keyPath: 'key'});
  });

  open_rq.onsuccess = t.step_func(event => {
    const store = db.transaction('store', 'readonly').objectStore('store');

    // Attempt to use an invalid key (null)
    assert_throws_dom('DataError', () => {
      store.get(null);
    });

    t.done();
  });
}, 'When an invalid key is used, throw DataError');
