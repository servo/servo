// Copyright 2016 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

var db_name = "db_wasm_test";
var obj_store = 'store';
var module_key = 'my_module';

function createAndSaveToIndexedDB(db_name) {
  return createWasmModule()
    .then(mod => new Promise((resolve, reject) => {
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
            db.close();
            reject(e);
            return;
          }
          tx.oncomplete = function() {
            db.close();
            resolve();
            return;
          };
        };
      };
    }));
}

function loadFromIndexedDB(db_name) {
  var open_request = indexedDB.open(db_name);

  return new Promise((resolve, reject) => {
    open_request.onsuccess = function() {
      var db = open_request.result;
      var tx = db.transaction(obj_store);
      var store = tx.objectStore(obj_store);
      var get_request = store.get(module_key);
      get_request.onsuccess = function() {
        var mod = get_request.result;
        db.close();
        assert_true(mod instanceof WebAssembly.Module);
        try {
          var instance = new WebAssembly.Instance(mod);
          resolve(instance.exports.increment(1));
        } catch(e) {
          reject(e);
        }
      };
      get_request.onerror = reject;
    };
  });
}

function TestIndexedDBLoadStoreSecure() {
  return createAndSaveToIndexedDB(db_name)
    .then(() => loadFromIndexedDB(db_name))
    .then(res => assert_equals(res, 2),
          error => assert_unreached(error));
}

function TestIndexedDBLoadStoreInsecure() {
  return createAndSaveToIndexedDB(db_name)
    .then(assert_unreached,
          error => {
            assert_true(error instanceof DOMException);
            assert_equals(error.name, 'DataCloneError');
          });
}

function SaveToIDBAndLoadInWorker() {
  return createAndSaveToIndexedDB(db_name)
  .then(() => {
    var worker = new Worker("wasm_idb_worker.js");
    return new Promise((resolve, reject) => {
      worker.onmessage = function (event) {
        if (typeof (event.data) == "string") {
          resolve(event.data);
          worker.terminate();
          worker = undefined;
        }
      };
      worker.postMessage({command: "load", db_name: db_name});
    })
  })
.then(data => assert_equals(data, "ok"),
    error => assert_unreached(error));
}

function SaveToIDBInWorkerAndLoadInMain() {
  var worker = new Worker("wasm_idb_worker.js");
  var ret = new Promise((resolve, reject) => {
    worker.onmessage = function (event) {
      if (typeof (event.data) == "string") {
        resolve(event.data);
        worker.terminate();
        worker = undefined;
      }
    };
  })
  worker.postMessage({command: "save", db_name: db_name});
  return ret
    .then(data => assert_equals(data, "ok"),
          error => assert_unreached(error))
    .then(() => loadFromIndexedDB(db_name))
    .then(res => assert_equals(res, 2),
          assert_unreached);
}
