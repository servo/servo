// META: title=StorageManager: estimate() for indexeddb
// META: script=/storage/buckets/resources/util.js

test(t => {
  assert_true('estimate' in navigator.storage);
  assert_equals(typeof navigator.storage.estimate, 'function');
  assert_true(navigator.storage.estimate() instanceof Promise);
}, 'estimate() method exists and returns a Promise');

promise_test(async t => {
  const estimate = await navigator.storage.estimate();
  assert_equals(typeof estimate, 'object');
  assert_true('usage' in estimate);
  assert_equals(typeof estimate.usage, 'number');
  assert_true('quota' in estimate);
  assert_equals(typeof estimate.quota, 'number');
}, 'estimate() resolves to dictionary with members');

promise_test(async t => {
  const arraySize = 1e6;
  const objectStoreName = "storageManager";
  const dbname =
      this.window ? window.location.pathname : 'estimate-worker.https.html';

  await indexedDbDeleteRequest(indexedDB, dbname);
  let estimate = await navigator.storage.estimate();

  const usageBeforeCreate = estimate.usage;
  const db =
      await indexedDbOpenRequest(t, indexedDB, dbname, (db_to_upgrade) => {
        db_to_upgrade.createObjectStore(objectStoreName);
      });

  estimate = await navigator.storage.estimate();
  const usageAfterCreate = estimate.usage;

  assert_greater_than(
    usageAfterCreate, usageBeforeCreate,
    'estimated usage should increase after object store is created');

  const txn = db.transaction(objectStoreName, 'readwrite');
  const buffer = new ArrayBuffer(arraySize);
  const view = new Uint8Array(buffer);

  for (let i = 0; i < arraySize; i++) {
    view[i] = Math.floor(Math.random() * 255);
  }

  const testBlob = new Blob([buffer], {type: 'binary/random'});
  txn.objectStore(objectStoreName).add(testBlob, 1);

  await transactionPromise(txn);

  estimate = await navigator.storage.estimate();
  const usageAfterPut = estimate.usage;
  assert_greater_than(
    usageAfterPut, usageAfterCreate,
    'estimated usage should increase after large value is stored');

  db.close();
}, 'estimate() shows usage increase after large value is stored');
