// META: title=StorageManager: estimate() for indexeddb

test(t => {
  assert_true('estimate' in navigator.storage);
  assert_equals(typeof navigator.storage.estimate, 'function');
  assert_true(navigator.storage.estimate() instanceof Promise);
}, 'estimate() method exists and returns a Promise');

promise_test(t => {
  return navigator.storage.estimate().then(result => {
    assert_true(typeof result === 'object');
    assert_true('usage' in result);
    assert_equals(typeof result.usage, 'number');
    assert_true('quota' in result);
    assert_equals(typeof result.quota, 'number');
  });
}, 'estimate() resolves to dictionary with members');

promise_test(t => {
  const arraySize = 1e6;
  const objectStoreName = "storageManager";
  const dbname = this.window ? window.location.pathname :
        "estimate-worker.https.html";

  let db;
  let usageBeforeCreate, usageAfterCreate, usageAfterPut;

  function deleteDB(name) {
    return new Promise((resolve, reject) => {
      let deleteRequest = indexedDB.deleteDatabase(name);
      deleteRequest.onerror = () => { reject(deleteRequest.error); };
      deleteRequest.onsuccess = () => { resolve(); };
    });
  }

  return deleteDB(dbname)
  .then(() => {
     return navigator.storage.estimate();
  })
  .then(estimate => {
    usageBeforeCreate = estimate.usage;
    return new Promise((resolve, reject) => {
      let openRequest = indexedDB.open(dbname);
      openRequest.onerror = () => { reject(openRequest.error); };
      openRequest.onupgradeneeded = event => {
        openRequest.result.createObjectStore(objectStoreName);
      };
      openRequest.onsuccess = () => { resolve(openRequest.result); };
    });
  })
  .then(connection => {
    db = connection;
    return navigator.storage.estimate();
  })
  .then(estimate => {
    usageAfterCreate = estimate.usage;
    assert_greater_than(
      usageAfterCreate, usageBeforeCreate,
      'estimated usage should increase after object store is created');

    let txn = db.transaction(objectStoreName, 'readwrite');
    let buffer = new ArrayBuffer(arraySize);
    let view = new Uint8Array(buffer);

    for (let i = 0; i < arraySize; i++) {
      view[i] = parseInt(Math.random() * 255);
    }

    let testBlob = new Blob([buffer], {type: "binary/random"});
    txn.objectStore(objectStoreName).add(testBlob, 1);

    return new Promise((resolve, reject) => {
      txn.onabort = () => { reject(txn.error); };
      txn.oncomplete = () => { resolve(); };
    });
  })
  .then(() => {
    return navigator.storage.estimate();
  })
  .then(estimate => {
    usageAfterPut = estimate.usage;
    assert_greater_than(
      usageAfterPut, usageAfterCreate,
      'estimated usage should increase after large value is stored');

    db.close();
    return deleteDB(dbname);
  })
  .then(() => {
    t.done();
  });
}, 'estimate() shows usage increase after large value is stored');
