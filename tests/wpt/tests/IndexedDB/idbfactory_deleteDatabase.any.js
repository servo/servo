// META: title=IDBFactory.deleteDatabase()
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  const delete_rq = indexedDB.deleteDatabase('db-that-doesnt-exist');
  delete_rq.onerror = fail(t, 'delete_rq.error');
  delete_rq.onsuccess = t.step_func(e => {
    assert_equals(e.oldVersion, 0, 'event.oldVersion');
    assert_equals(e.target.source, null, 'event.target.source');
  });

  const open_rq = createdb(t, undefined, 9);
  open_rq.onupgradeneeded = t.step_func(e => {});
  open_rq.onsuccess = t.step_func(e => {
    const db = e.target.result;
    db.close();

    const delete_rq1 = indexedDB.deleteDatabase(db.name);
    delete_rq1.onerror = fail(t, 'delete_rq1.error');
    delete_rq1.onsuccess = t.step_func(e => {
      assert_equals(e.oldVersion, 9, 'event.oldVersion');
      assert_equals(e.target.source, null, 'event.target.source');
    });

    const delete_rq2 = indexedDB.deleteDatabase(db.name);
    delete_rq2.onerror = fail(t, 'delete_rq2.error');

    delete_rq2.onsuccess = t.step_func_done(e => {
      assert_equals(e.oldVersion, 0, 'event.oldVersion');
      assert_equals(e.target.source, null, 'event.target.source');
    });
  });
}, 'deleteDatabase() request should have no source, and deleting a non-existent\
 database should succeed with oldVersion of 0.');

async_test(t => {
  const open_rq = createdb(t, undefined, 9);

  open_rq.onupgradeneeded = t.step_func(e => {});

  open_rq.onsuccess = t.step_func(e => {
    const db = e.target.result;
    db.close();

    const delete_rq = indexedDB.deleteDatabase(db.name);
    delete_rq.onerror = t.step_func(e => {
      assert_unreached('Unexpected delete_rq.error event');
    });

    delete_rq.onsuccess = t.step_func(e => {
      assert_equals(e.target.result, undefined, 'result');
      t.done();
    });
  });
}, 'Result of the deleteDatabase() request is set to undefined.');

async_test(t => {
  let db;
  const open_rq = createdb(t, undefined, 9);

  open_rq.onupgradeneeded = t.step_func(e => {
    db = e.target.result;
    db.createObjectStore('os');
  });

  open_rq.onsuccess = t.step_func(e => {
    db.close();

    const delete_rq = indexedDB.deleteDatabase(db.name);
    delete_rq.onerror = t.step_func(e => {
      assert_unreached('Unexpected delete_rq.error event');
    });

    delete_rq.onsuccess = t.step_func(e => {
      assert_equals(e.oldVersion, 9, 'oldVersion');
      assert_equals(e.newVersion, null, 'newVersion');
      assert_equals(e.target.result, undefined, 'result');
      assert_true(
          e instanceof IDBVersionChangeEvent,
          'e instanceof IDBVersionChangeEvent');
      t.done();
    });
  });
}, 'The deleteDatabase() request\'s success event is an IDBVersionChangeEvent.');

async_test(t => {
  const dbname = location + '-' + t.name;

  indexedDB.deleteDatabase(dbname);

  let db;
  const openrq = indexedDB.open(dbname, 3);

  openrq.onupgradeneeded = t.step_func(e => {
    e.target.result.createObjectStore('store');
  });

  openrq.onsuccess = t.step_func(e => {
    db = e.target.result;

    // Errors
    db.onversionchange = fail(t, 'db.versionchange');
    db.onerror = fail(t, 'db.error');
    db.abort = fail(t, 'db.abort');

    step_timeout(t.step_func(() => Second(t, dbname)), 4);
    db.close();
  });

  // Errors
  openrq.onerror = fail(t, 'open.error');
  openrq.onblocked = fail(t, 'open.blocked');
}, 'Delete an existing database - Test events opening a second \
database when one connection is open already');

function Second(t, dbname) {
  const deleterq = indexedDB.deleteDatabase(dbname);

  deleterq.onsuccess = e => {
    t.done();
  };

  deleterq.onerror = fail(t, 'delete.error');
  deleterq.onblocked = fail(t, 'delete.blocked');
  deleterq.onupgradeneeded = fail(t, 'delete.upgradeneeded');
}
