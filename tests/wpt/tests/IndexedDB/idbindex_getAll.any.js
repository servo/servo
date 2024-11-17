// META: title=IndexedDB: Test IDBIndex.getAll
// META: global=window,worker
// META: script=resources/nested-cloning-common.js
// META: script=resources/support.js
// META: script=resources/support-get-all.js
// META: script=resources/support-promises.js

'use_strict';

function createGetAllRequest(t, storeName, connection, range, maxCount) {
    const transaction = connection.transaction(storeName, 'readonly');
    const store = transaction.objectStore(storeName);
    const index = store.index('test_idx');
    const req = index.getAll(range, maxCount);
    req.onerror = t.unreached_func('getAll request should succeed');
    return req;
}

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection, 'C');
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), ['c']);
          assert_array_equals(data.map(function(e) { return e.upper; }), ['C']);
          t.done();
      });
    },
    'Single item get');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'empty', connection);
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result, [],
              'getAll() on empty object store should return an empty array');
          t.done();
      });
    },
    'Empty object store');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), alphabet);
          assert_array_equals(data.map(function(e) { return e.upper; }), ALPHABET);
          t.done();
      });
    },
    'Get all');

index_get_all_test((test, connection) => {
  const request = createGetAllRequest(test, 'large-values', connection);
  request.onsuccess = test.step_func(event => {
    const actualResults = event.target.result;
    assert_true(Array.isArray(actualResults), 'The results must be an array');

    const expectedRecords = expectedIndexRecords['large-values'];
    assert_equals(
        actualResults.length, expectedRecords.length,
        'The results array must contain the expected number of records');

    // Verify each value that must contain `{ seed, randomValue }`.
    for (let i = 0; i < expectedRecords.length; i++) {
      assert_equals(
          actualResults[i].seed, expectedRecords[i].value.seed,
          'The results must contain the expected seed');

      assert_large_array_equals(
          actualResults[i].randomValue, expectedRecords[i].value.randomValue,
          'The results must contain the expected value');
    }
    test.done();
  });
}, 'Get all with large values');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection, undefined,
                                    10);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'abcdefghij'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'ABCDEFGHIJ'.split(''));
          t.done();
      });
    },
    'maxCount=10');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
                                    IDBKeyRange.bound('G', 'M'));
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_array_equals(data.map(function(e) { return e.ch; }), 'ghijklm'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'GHIJKLM'.split(''));
          t.done();
      });
    },
    'Get bound range');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
                                    IDBKeyRange.bound('G', 'M'), 3);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'ghi'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'GHI'.split(''));
          t.done();
      });
    },
    'Get bound range with maxCount');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
          IDBKeyRange.bound('G', 'K', false, true));
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'ghij'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'GHIJ'.split(''));
          t.done();
      });
    },
    'Get upper excluded');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
          IDBKeyRange.bound('G', 'K', true, false));
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'hijk'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'HIJK'.split(''));
          t.done();
      });
    },
    'Get lower excluded');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'generated',
          connection, IDBKeyRange.bound(4, 15), 3);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_true(Array.isArray(data));
          assert_equals(data.length, 0);
          t.done();
      });
    },
    'Get bound range (generated) with maxCount');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line',
          connection, "Doesn't exist");
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result, [],
              'getAll() using a nonexistent key should return an empty array');
          t.done();
      });
    },
    'Non existent key');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
          undefined, 0);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), alphabet);
          assert_array_equals(data.map(function(e) { return e.upper; }), ALPHABET);
          t.done();
      });
    },
    'maxCount=0');

index_get_all_test(function(test, connection) {
  const request = createGetAllRequest(
      test, 'out-of-line', connection,
      /*query=*/ undefined, /*count=*/ 4294967295);
  request.onsuccess = test.step_func(function(event) {
    const data = event.target.result;
    assert_class_string(data, 'Array', 'result should be an array');
    assert_array_equals(
        data.map(function(e) {
          return e.ch;
        }),
        alphabet);
    assert_array_equals(
        data.map(function(e) {
          return e.upper;
        }),
        ALPHABET);
    test.done();
  });
}, 'Max value count');

index_get_all_test((test, connection) => {
  const request = createGetAllRequest(
      test, /*storeName=*/ 'out-of-line', connection,
      IDBKeyRange.upperBound('0'));
  request.onsuccess = test.step_func((event) => {
    assert_array_equals(
        event.target.result, /*expectedResults=*/[],
        'getAll() with an empty query range must return an empty array');
    test.done();
  });
}, 'Query with empty range where  first key < upperBound');

index_get_all_test((test, connection) => {
  const request = createGetAllRequest(
      test, /*storeName=*/ 'out-of-line', connection,
      IDBKeyRange.lowerBound('ZZ'));
  request.onsuccess = test.step_func((event) => {
    assert_array_equals(
        event.target.result, /*expectedResults=*/[],
        'getAll() with an empty query range must return an empty array');
    test.done();
  });
}, 'Query with empty range where lowerBound < last key');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line-not-unique', connection,
                                    'first');
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'abcdefghijklm'.split(''));
          assert_true(data.every(function(e) { return e.half === 'first'; }));
          t.done();
      });
    },
    'Retrieve multiEntry key');

index_get_all_test(
    function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line-multi', connection,
                                    'vowel');
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), ['a', 'e', 'i', 'o', 'u']);
          assert_array_equals(data[0].attribs, ['vowel', 'first']);
          assert_true(data.every(function(e) { return e.attribs[0] === 'vowel'; }));
          t.done();
      });
    },
    'Retrieve one key multiple values');
