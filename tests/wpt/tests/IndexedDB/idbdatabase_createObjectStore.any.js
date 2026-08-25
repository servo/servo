// META: title=IDBDatabase.createObjectStore()
// META: global=window,worker
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbdatabase-createobjectstore

'use strict';

async_test(t => {
  let db;
  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    let store = db.createObjectStore('');

    for (let i = 0; i < 5; i++)
      store.add({idx: 'object_' + i}, i);

    store.createIndex('', 'idx');

    store.get(4).onsuccess = t.step_func(function(e) {
      assert_equals(e.target.result.idx, 'object_4', 'result');
    });
    assert_equals(store.indexNames[0], '', 'indexNames[0]');
    assert_equals(store.indexNames.length, 1, 'indexNames.length');
  };

  open_rq.onsuccess = function() {
    let store = db.transaction('').objectStore('');

    assert_equals(store.indexNames[0], '', 'indexNames[0]');
    assert_equals(store.indexNames.length, 1, 'indexNames.length');

    store.index('').get('object_4').onsuccess = t.step_func(function(e) {
      assert_equals(e.target.result.idx, 'object_4', 'result');
      t.done();
    });
  };
}, 'Both with empty name');

async_test(t => {
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    let db = e.target.result;
    let objStore = db.createObjectStore('instancetest');

    assert_true(
        objStore instanceof IDBObjectStore, 'instanceof IDBObjectStore');
  };

  open_rq.onsuccess = function(e) {
    let db = e.target.result;
    let objStore =
        db.transaction('instancetest', 'readonly').objectStore('instancetest');

    assert_true(
        objStore instanceof IDBObjectStore, 'instanceof IDBObjectStore');
    t.done();
  };
}, 'Returns an instance of IDBObjectStore');

async_test(t => {
  let db;
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    let store;
    let i;
    for (i = 0; i < 1000; i++) {
      store = db.createObjectStore('object_store_' + i);
      store.add('test', 1);
    }

    store.get(1).onsuccess = t.step_func(function(e) {
      assert_equals(e.target.result, 'test');
    });
  };
  open_rq.onsuccess = function(e) {
    db.close();
    self.indexedDB.deleteDatabase(db.name).onsuccess = function(e) {
      t.done();
    }
  };
}, 'Create 1000 object stores, add one item and delete');

async_test(t => {
  let db;
  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    let store = db.createObjectStore('');

    for (let i = 0; i < 5; i++)
      store.add('object_' + i, i);

    assert_equals(db.objectStoreNames[0], '', 'db.objectStoreNames[0]');
    assert_equals(db.objectStoreNames.length, 1, 'objectStoreNames.length');
  };

  open_rq.onsuccess = function() {
    let store = db.transaction('').objectStore('');

    store.get(2).onsuccess = t.step_func(function(e) {
      assert_equals(e.target.result, 'object_2');
    })

    assert_equals(db.objectStoreNames[0], '', 'db.objectStoreNames[0]');
    assert_equals(db.objectStoreNames.length, 1, 'objectStoreNames.length');

    t.done();
  };
}, 'Empty name');

async_test(t => {
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    let db = e.target.result;
    db.createObjectStore('store');
    assert_throws_dom('ConstraintError', function() {
      db.createObjectStore('store', {
        keyPath: 'key1',
      });
    });
    t.done();
  };
}, 'Attempting to create an existing object store with a different keyPath throw ConstraintError ');

async_test(t => {
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    let db = e.target.result;
    let objStore = db.createObjectStore('prop', {keyPath: 'mykeypath'});

    assert_equals(objStore.name, 'prop', 'object store name');
    assert_equals(objStore.keyPath, 'mykeypath', 'key path');
    assert_equals(objStore.autoIncrement, false, 'auto increment');
  };

  open_rq.onsuccess = function(e) {
    let db = e.target.result;
    let objStore = db.transaction('prop', 'readonly').objectStore('prop');

    assert_equals(objStore.name, 'prop', 'object store name');
    assert_equals(objStore.keyPath, 'mykeypath', 'key path');
    assert_equals(objStore.autoIncrement, false, 'auto increment');
    t.done();
  };
}, 'Object store \'name\' and \'keyPath\' properties are correctly set ');

async_test(t => {
  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function() {};
  open_rq.onsuccess = function(e) {
    let db = e.target.result;
    assert_throws_dom('InvalidStateError', function() {
      db.createObjectStore('fails')
    });
    t.done();
  };
}, 'Attempt to create an object store outside of a version change transaction ');

async_test(t => {
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    let db = e.target.result;
    db.createObjectStore('dupe');
    assert_throws_dom('ConstraintError', function() {
      db.createObjectStore('dupe');
    });

    // Bonus test creating a new objectstore after the exception
    db.createObjectStore('dupe ');
    t.done();
  };
}, 'Attempt to create an object store that already exists ');

async_test(t => {
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    let db = e.target.result;

    db.createObjectStore('My cool object store name');
    assert_true(
        db.objectStoreNames.contains('My cool object store name'),
        'objectStoreNames.contains');
  };

  open_rq.onsuccess = function(e) {
    let db = e.target.result;

    assert_true(
        db.objectStoreNames.contains('My cool object store name'),
        'objectStoreNames.contains (in success)');
    t.done();
  };
}, 'Object store\'s name appears in database\'s list ');

async_test(t => {
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    let db = e.target.result;

    assert_throws_dom('SyntaxError', function() {
      db.createObjectStore('invalidkeypath', {keyPath: 'Invalid Keypath'})
    });

    assert_throws_dom('SyntaxError', function() {
      db.createObjectStore(
          'invalidkeypath', {autoIncrement: true, keyPath: 'Invalid Keypath'})
    });

    t.done();
  };
}, 'Attempt to create an object store with an invalid key path ');

async_test(t => {
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    let db = e.target.result;
    db.createObjectStore('with unknown param', {parameter: 0});

    t.done();
  };
}, 'Create an object store with an unknown optional parameter ');

function optionalParameters(desc, params, t) {
  promise_test(t => {
    return new Promise((resolve, reject) => {
      const request = createdb(t);
      request.onupgradeneeded = t.step_func(function(e) {
        e.target.result.createObjectStore('store', params);
        resolve();
      });
    });
  }, desc);
}

optionalParameters('autoInc true', {autoIncrement: true});

optionalParameters(
    'autoInc true, keyPath null', {autoIncrement: true, keyPath: null});

optionalParameters(
    'autoInc true, keyPath undefined',
    {autoIncrement: true, keyPath: undefined});

optionalParameters(
    'autoInc true, keyPath string', {autoIncrement: true, keyPath: 'a'});

optionalParameters(
    'autoInc false, keyPath empty', {autoIncrement: false, keyPath: ''});

optionalParameters(
    'autoInc false, keyPath array',
    {autoIncrement: false, keyPath: ['h', 'j']});

optionalParameters(
    'autoInc false, keyPath string', {autoIncrement: false, keyPath: 'abc'});

optionalParameters('keyPath empty', {keyPath: ''});

optionalParameters('keyPath array', {keyPath: ['a', 'b']});

optionalParameters('keyPath string', {keyPath: 'abc'});

optionalParameters('keyPath null', {keyPath: null});

optionalParameters('keyPath undefined', {keyPath: undefined});

function invalid_optionalParameters(
    desc, params, exception = 'InvalidAccessError') {
  promise_test(t => {
    return new Promise((resolve, reject) => {
      const request = createdb(t);
      request.onupgradeneeded = t.step_func(function(e) {
        assert_throws_dom(exception, function() {
          e.target.result.createObjectStore('store', params);
        });
        resolve();
      });
    });
  }, desc);
}

invalid_optionalParameters(
    'autoInc and empty keyPath', {autoIncrement: true, keyPath: ''});

invalid_optionalParameters(
    'autoInc and keyPath array', {autoIncrement: true, keyPath: []},
    'SyntaxError');

invalid_optionalParameters(
    'autoInc and keyPath array 2', {autoIncrement: true, keyPath: ['hey']});

invalid_optionalParameters(
    'autoInc and keyPath object',
    {autoIncrement: true, keyPath: {a: 'hey', b: 2}}, 'SyntaxError');
