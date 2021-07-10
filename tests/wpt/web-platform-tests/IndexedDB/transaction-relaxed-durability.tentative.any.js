// META: script=support-promises.js
// META: timeout=long

/**
 * This file contains the webplatform smoke tests for the optional
 * durability parameter of the IndexedDB transaction API.
 *
 * @author enne@chromium.org
 */

// Smoke test optional parameter on IndexedDB.transaction.
let cases = [
  { options: undefined, expected: 'default' },
  { options: {}, expected: 'default' },
  { options: { durability: 'default'}, expected: 'default' },
  { options: { durability: 'relaxed'}, expected: 'relaxed' },
  { options: { durability: 'strict'}, expected: 'strict' },
];

for (let i = 0; i < cases.length; ++i) {
  promise_test(async testCase => {
    const db = await createDatabase(testCase, db => {
      createBooksStore(testCase, db);
    });
    const txn = db.transaction(['books'], 'readwrite', cases[i].options);
    const objectStore = txn.objectStore('books');
    objectStore.put({isbn: 'one', title: 'title1'});
    await promiseForTransaction(testCase, txn);

    assert_equals(txn.durability, cases[i].expected);

    const txn2 = db.transaction(['books'], 'readonly');
    const objectStore2 = txn2.objectStore('books');
    const getTitle1 = objectStore2.get('one');
    await promiseForTransaction(testCase, txn2);
    assert_array_equals(
        [getTitle1.result.title],
        ['title1'],
        'The title should match that which was put.');
    db.close();
  }, 'Committed data can be read back out: case ' + i);
}

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    createBooksStore(testCase, db);
  });

  assert_throws_js(TypeError, function() {
      db.transaction(['books'], 'readwrite', { durability: 'invalid' });
  });
  db.close();
}, 'Invalid durability option throws a TypeError');
