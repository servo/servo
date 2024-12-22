// META: title=IndexedDB: Test IDBObjectStore.getAllRecords
// META: global=window,worker
// META: script=resources/nested-cloning-common.js
// META: script=resources/support.js
// META: script=resources/support-get-all.js
// META: script=resources/support-promises.js
// META: timeout=long

'use strict';

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {query: /*key=*/ 'c'},
    'Single item');

object_store_get_all_records_test(
    /*storeName=*/ 'generated', /*options=*/ {query: /*key=*/ 3},
    'Single item with generated key');

object_store_get_all_records_test(
    /*storeName=*/ 'empty', /*options=*/ undefined, 'Empty object store');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ undefined, 'Get all records');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {},
    'Get all records with empty options');

object_store_get_all_records_test(
    /*storeName=*/ 'large-values', /*options=*/ undefined,
    'Get all records with large values');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {count: 10}, 'Count');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {query: IDBKeyRange.bound('g', 'm')},
    'Query with bound range');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {query: IDBKeyRange.bound('g', 'm'), count: 3},
    'Query with bound range and count');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {
      query:
          IDBKeyRange.bound('g', 'k', /*lowerOpen=*/ false, /*upperOpen=*/ true)
    },
    'Query with upper excluded bound range');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {
      query:
          IDBKeyRange.bound('g', 'k', /*lowerOpen=*/ true, /*upperOpen=*/ false)
    },
    'Query with lower excluded bound range');

object_store_get_all_records_test(
    /*storeName=*/ 'generated',
    /*options=*/ {query: IDBKeyRange.bound(4, 15), count: 3},
    'Query with bound range and count for generated keys');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {query: 'Doesn\'t exist'},
    'Query with nonexistent key');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {count: 0}, 'Zero count');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {count: 4294967295},
    'Max value count');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {query: IDBKeyRange.upperBound('0')},
    'Query with empty range where first key < upperBound');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {query: IDBKeyRange.lowerBound('zz')},
    'Query with empty range where lowerBound < last key');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {direction: 'next'},
    'Direction: next');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {direction: 'prev'},
    'Direction: prev');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {direction: 'nextunique'},
    'Direction: nextunique');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {direction: 'prevunique'},
    'Direction: prevunique');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {
      direction: 'prev',
      query: IDBKeyRange.bound('b', 'x'),
    },
    'Direction and query');

object_store_get_all_records_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {
      direction: 'prev',
      query: IDBKeyRange.bound('b', 'x'),
      count: 4
    },
    'Direction, query and count');

object_store_get_all_test_setup(
    /*storeName=*/ 'out-of-line', (test, connection, expectedRecords) => {
      const transaction = connection.transaction('out-of-line', 'readonly');
      const store = transaction.objectStore('out-of-line');
      const request = store.getAllRecords();
      transaction.commit();

      transaction.oncomplete =
          test.unreached_func('transaction completed before request succeeded');

      request.onerror =
          test.unreached_func('getAllRecords request must  succeed');

      request.onsuccess = test.step_func((event) => {
        const actualResults = event.target.result;
        assert_records_equals(actualResults, expectedRecords);
        test.done();
      });
    }, 'Get all records with transaction.commit()');
