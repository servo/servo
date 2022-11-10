// META: title=Buckets API: Tests for indexedDB API.
// META: global=window,worker
// META: script=resources/support-promises.js

promise_test(async testCase => {
  const inboxBucket = await navigator.storageBuckets.open('inbox_bucket');
  testCase.add_cleanup(() => {
    navigator.storageBuckets.delete('inbox_bucket');
  });
  const outboxBucket = await navigator.storageBuckets.open('outbox_bucket');
  testCase.add_cleanup(() => {
    navigator.storageBuckets.delete('outbox_bucket');
  });

  // Set up similar databases in two buckets.
  const inboxDb = await new Promise(resolve => {
    const request = inboxBucket.indexedDB.open('messages');
    request.onupgradeneeded = (event) => {
      const inboxStore =
          event.target.result.createObjectStore('primary', {keyPath: 'id'});
      event.target.transaction.commit();
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });

  const txn = inboxDb.transaction(['primary'], 'readwrite');
  const inboxStore = txn.objectStore('primary');
  inboxStore.put({ subject: 'Bonjour', id: '42'});
  txn.commit();
  await promiseForTransaction(testCase, txn);

  const outboxDb = await new Promise(resolve => {
    const request = outboxBucket.indexedDB.open('messages');
    request.onupgradeneeded = (event) => {
      const outboxStore =
          event.target.result.createObjectStore('primary', {keyPath: 'id'});
      event.target.transaction.commit();
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });

  const txn2 = outboxDb.transaction(['primary'], 'readwrite');
  const outboxStore = txn2.objectStore('primary');
  outboxStore.put({ subject: 're: Bonjour', id: '47'});
  txn2.commit();
  await promiseForTransaction(testCase, txn2);

  // Make sure it's possible to read from the bucket database.
  const inboxMessage = await new Promise(resolve => {
    const txn3 = inboxDb.transaction(['primary'], 'readonly');
    const inboxLookup = txn3.objectStore('primary').get('42');
    inboxLookup.onsuccess = (e) => resolve(inboxLookup.result);
    inboxLookup.onerror = (e) => reject(inboxLookup.error);
  });
  assert_equals(inboxMessage.subject, 'Bonjour');

  // Make sure it's possible to read from the other bucket database.
  const outboxMessage = await new Promise(resolve => {
    const txn4 = outboxDb.transaction(['primary'], 'readonly');
    const outboxLookup = txn4.objectStore('primary').get('47');
    outboxLookup.onsuccess = (e) => resolve(outboxLookup.result);
    outboxLookup.onerror = (e) => reject(outboxLookup.error);
  });
  assert_equals(outboxMessage.subject, 're: Bonjour');

  // Make sure they are different databases (looking up the data keyed on `47`
  // fails in the first database).
  const nonexistentInboxMessage = await new Promise(resolve => {
    const txn5 = inboxDb.transaction(['primary'], 'readonly');
    const nonexistentInboxLookup = txn5.objectStore('primary').get('47');
    nonexistentInboxLookup.onsuccess = (e) =>
        resolve(nonexistentInboxLookup.result);
    nonexistentInboxLookup.onerror = (e) =>
        reject(nonexistentInboxLookup.error);
  });
  assert_equals(nonexistentInboxMessage, undefined);
}, 'Basic test that buckets create independent databases.');
