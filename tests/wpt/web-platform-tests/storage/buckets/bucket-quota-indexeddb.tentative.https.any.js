// META: title=Bucket quota enforcement for indexeddb
// META: script=/storage/buckets/resources/util.js

promise_test(async t => {
  const arraySize = 1e6;
  const objectStoreName = "storageManager";
  const dbname =
      this.window ? window.location.pathname : 'estimate-worker.https.html';

  let quota = arraySize / 2;
  const bucket = await navigator.storageBuckets.open('idb', {quota});

  await indexedDbDeleteRequest(bucket.indexedDB, dbname);

  const db =
      await indexedDbOpenRequest(t, bucket.indexedDB, dbname, (db_to_upgrade) => {
        db_to_upgrade.createObjectStore(objectStoreName);
      });

  const txn = db.transaction(objectStoreName, 'readwrite');
  const buffer = new ArrayBuffer(arraySize);
  const view = new Uint8Array(buffer);

  for (let i = 0; i < arraySize; i++) {
    view[i] = Math.floor(Math.random() * 255);
  }

  const testBlob = new Blob([buffer], {type: 'binary/random'});
  txn.objectStore(objectStoreName).add(testBlob, 1);

  await promise_rejects_dom(
      t, 'QuotaExceededError', transactionPromise(txn));

  db.close();
}, 'IDB respects bucket quota');
