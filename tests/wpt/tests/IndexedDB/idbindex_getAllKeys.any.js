// META: title=IndexedDB: Test IDBIndex.getAllKeys.
// META: global=window,worker
// META: script=resources/nested-cloning-common.js
// META: script=resources/support.js
// META: script=resources/support-get-all.js
// META: script=resources/support-promises.js

'use_strict';

function createGetAllKeysRequest(t, storeName, connection, range, maxCount) {
    const transaction = connection.transaction(storeName, 'readonly');
    const store = transaction.objectStore(storeName);
    const index = store.index('test_idx');
    const req = index.getAllKeys(range, maxCount);
    req.onerror = t.unreached_func('getAllKeys request should succeed');
    return req;
}

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'out-of-line', connection, 'C');
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_array_equals(evt.target.result, ['c']);
          t.done();
      });
    },
    'Single item get');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'empty', connection);
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result, [],
              'getAllKeys() on empty object store should return empty array');
          t.done();
      });
    },
    'Empty object store');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'out-of-line', connection);
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result, alphabet,
              'getAllKeys() should return a..z');
          t.done();
      });
    },
    'Get all keys');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'generated', connection);
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result,
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
               19, 20, 21, 22, 23, 24, 25, 26],
              'getAllKeys() should return 1..26');
          t.done();
      });
    },
    'Get all generated keys');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'out-of-line', connection, undefined,
                                    10);
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result,
                             'abcdefghij'.split(''),
                             'getAllKeys() should return a..j');
          t.done();
      });
    },
    'maxCount=10');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'out-of-line', connection,
                                    IDBKeyRange.bound('G', 'M'));
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result,
                              'ghijklm'.split(''),
                              'getAllKeys() should return g..m');
          t.done();
      });
    },
    'Get bound range');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'out-of-line', connection,
                                    IDBKeyRange.bound('G', 'M'), 3);
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result,
                             ['g', 'h', 'i'],
                             'getAllKeys() should return g..i');
          t.done();
      });
    },
    'Get bound range with maxCount');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'out-of-line', connection,
          IDBKeyRange.bound('G', 'K', false, true));
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result,
                             ['g', 'h', 'i', 'j'],
                             'getAllKeys() should return g..j');
          t.done();
      });
    },
    'Get upper excluded');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'out-of-line', connection,
          IDBKeyRange.bound('G', 'K', true, false));
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result,
                             ['h', 'i', 'j', 'k'],
                             'getAllKeys() should return h..k');
          t.done();
      });
    },
    'Get lower excluded');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'generated',
          connection, IDBKeyRange.bound(4, 15), 3);
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result, [],
                              'getAllKeys() should return []');
          t.done();
      });
    },
    'Get bound range (generated) with maxCount');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'out-of-line',
          connection, "Doesn't exist");
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result, [],
              'getAllKeys() using a nonexistent key should return empty array');
          t.done();
      });
    },
    'Non existent key');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllKeysRequest(t, 'out-of-line', connection,
          undefined, 0);
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result, alphabet,
              'getAllKeys() should return a..z');
          t.done();
      });
    },
    'maxCount=0');

index_get_all_test(function(test, connection) {
  const request = createGetAllKeysRequest(
      test, 'out-of-line', connection,
      /*query=*/ undefined, /*count=*/ 4294967295);
  request.onsuccess = test.step_func(function(event) {
    assert_array_equals(
        event.target.result, alphabet,
        'getAllKeys() must return an array containing all keys');
    test.done();
  });
}, 'Max value count');

index_get_all_test((test, connection) => {
  const request = createGetAllKeysRequest(
      test, /*storeName=*/ 'out-of-line', connection,
      IDBKeyRange.upperBound('0'));
  request.onsuccess = test.step_func((event) => {
    assert_array_equals(
        event.target.result, /*expectedResults=*/[],
        'getAllKeys() with an empty query range must return an empty array');
    test.done();
  });
}, 'Query with empty range where  first key < upperBound');

index_get_all_test((test, connection) => {
  const request = createGetAllKeysRequest(
      test, /*storeName=*/ 'out-of-line', connection,
      IDBKeyRange.lowerBound('ZZ'));
  request.onsuccess = test.step_func((event) => {
    assert_array_equals(
        event.target.result, /*expectedResults=*/[],
        'getAllKeys() with an empty query range must return an empty array');
    test.done();
  });
}, 'Query with empty range where lowerBound < last key');

index_get_all_test(function(t, connection) {
  const req =
      createGetAllKeysRequest(t, 'out-of-line-multi', connection, 'vowel');
  req.onsuccess = t.step_func(function(evt) {
    assert_array_equals(evt.target.result, ['a', 'e', 'i', 'o', 'u'])
    t.done();
  });
}, 'Retrieve multiEntry keys');
