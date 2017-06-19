// Copyright 2016 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

var db_name = 'db';
var obj_store = 'store';
var module_key = 'my_module';

function createAndSaveToIndexedDB() {
  return new Promise((resolve, reject) => {
    createWasmModule()
      .then(mod => {
        var delete_request = indexedDB.deleteDatabase(db_name);
        delete_request.onsuccess = function() {
          var open_request = indexedDB.open(db_name);
          open_request.onupgradeneeded = function() {
            var db = open_request.result;
            db.createObjectStore(obj_store);
          };
          open_request.onsuccess = function() {
            var db = open_request.result;
            var tx = db.transaction(obj_store, 'readwrite');
            var store = tx.objectStore(obj_store);
            try {
              store.put(mod, module_key);
            } catch(e) {
              reject(e);
              return;
            }
            tx.oncomplete = function() {
              resolve();
            };
            tx.onabort = function() {
              reject(transaction.error);
            };
          };
        };
      })
      .catch(error => reject(error));
  });
}

function loadFromIndexedDB(prev) {
  return new Promise((resolve, reject) => {
    prev.then(() => {
      var open_request = indexedDB.open(db_name);
      open_request.onsuccess = function() {
        var db = open_request.result;
        var tx = db.transaction(obj_store);
        var store = tx.objectStore(obj_store);
        var get_request = store.get(module_key);
        get_request.onsuccess = function() {
          var mod = get_request.result;
          assert_true(mod instanceof WebAssembly.Module);
          try {
            var instance = new WebAssembly.Instance(mod);
          } catch(e) {
            reject(e);
            return;
          }
          resolve(instance.exports.increment(1));
        };
      };
    });
  });
}

function TestIndexedDBLoadStoreSecure() {
  return loadFromIndexedDB(createAndSaveToIndexedDB())
    .then(res => assert_equals(res, 2),
          error => assert_unreached(error));
}

function TestIndexedDBLoadStoreInsecure() {
  return createAndSaveToIndexedDB()
    .then(assert_unreached,
          error => {
            assert_true(error instanceof DOMException);
            assert_equals(error.name, 'DataCloneError');
          });
}
