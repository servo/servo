// META: global=window,worker
// META: script=resources/support.js
// META: title=IDBIndex.multiEntry

'use strict';

async_test(t => {
  createdb(t).onupgradeneeded = function(e) {
    let store = e.target.result.createObjectStore('store');
    assert_throws_dom('InvalidAccessError', function() {
      store.createIndex('actors', ['name'], {multiEntry: true})
    });
    t.done();
  };
}, 'Array keyPath with multiEntry');

async_test(t => {
  let db;
  let open_rq = createdb(t);
  let obj = {test: 'yo', idxkeys: []};
  for (let i = 0; i < 1000; i++)
    obj.idxkeys.push('index_no_' + i);
  open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    db.createObjectStore('store').createIndex(
        'index', 'idxkeys', {multiEntry: true});
  };
  open_rq.onsuccess = function(e) {
    let tx = db.transaction('store', 'readwrite', {durability: 'relaxed'});
    tx.objectStore('store').put(obj, 1).onsuccess = t.step_func(function(e) {
      assert_equals(e.target.result, 1, 'put\'d key');
    });
    tx.oncomplete = t.step_func(function() {
      let idx = db.transaction('store', 'readonly', {durability: 'relaxed'})
                    .objectStore('store')
                    .index('index');
      for (let i = 0; i < 1000; i++) {
        idx.get('index_no_' + i).onsuccess = t.step_func(function(e) {
          assert_equals(e.target.result.test, 'yo');
        });
      }

      idx.get('index_no_999').onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result.test, 'yo');
        assert_equals(e.target.result.idxkeys.length, 1000);
        t.done();
      });
    });
  };
}, 'A 1000 entry multiEntry array');

async_test(t => {
  let db;
  let expected_keys = [1, 2, 2, 3, 3];
  let open_rq = createdb(t)
  open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    let store = db.createObjectStore('store')
    store.createIndex('actors', 'name', {multiEntry: true})
    store.add({name: 'Odin'}, 1);
    store.add({name: ['Rita', 'Scheeta', {Bobby: 'Bobby'}]}, 2);
    store.add({name: [{s: 'Robert'}, 'Neil', 'Bobby']}, 3);
  };
  open_rq.onsuccess = function(e) {
    let gotten_keys = [];
    let idx = db.transaction('store', 'readonly', {durability: 'relaxed'})
                  .objectStore('store')
                  .index('actors');
    idx.getKey('Odin').onsuccess = t.step_func(function(e) {
      gotten_keys.push(e.target.result)
    });
    idx.getKey('Rita').onsuccess = t.step_func(function(e) {
      gotten_keys.push(e.target.result)
    });
    idx.getKey('Scheeta').onsuccess = t.step_func(function(e) {
      gotten_keys.push(e.target.result)
    });
    idx.getKey('Neil').onsuccess = t.step_func(function(e) {
      gotten_keys.push(e.target.result)
    });
    idx.getKey('Bobby').onsuccess = t.step_func(function(e) {
      gotten_keys.push(e.target.result)
      assert_array_equals(gotten_keys, expected_keys);
      t.done();
    });
  }
}, 'Adding keys');
