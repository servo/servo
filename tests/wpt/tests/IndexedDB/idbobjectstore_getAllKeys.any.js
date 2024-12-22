// META: title=IndexedDB: Test IDBObjectStore.getAllKeys
// META: global=window,worker
// META: script=resources/nested-cloning-common.js
// META: script=resources/support.js
// META: script=resources/support-get-all.js
// META: script=resources/support-promises.js
// META: timeout=long

'use strict';

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {query: 'c'}, 'Single item get');

object_store_get_all_keys_test(
    /*storeName=*/ 'generated', /*options=*/ {query: 3},
    'Single item get (generated key)');

object_store_get_all_keys_test(
    /*storeName=*/ 'empty', /*options=*/ undefined,
    'getAllKeys on empty object store');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line', /*options=*/ undefined, 'Get all keys');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {count: 10}, 'Test maxCount');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {query: IDBKeyRange.bound('g', 'm')}, 'Get bound range');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {query: IDBKeyRange.bound('g', 'm'), count: 3},
    'Get bound range with maxCount');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {
      query:
          IDBKeyRange.bound('g', 'k', /*lowerOpen=*/ false, /*upperOpen=*/ true)
    },
    'Get upper excluded');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line', /*options=*/ {
      query:
          IDBKeyRange.bound('g', 'k', /*lowerOpen=*/ true, /*upperOpen=*/ false)
    },
    'Get lower excluded');

object_store_get_all_keys_test(
    /*storeName=*/ 'generated',
    /*options=*/ {query: IDBKeyRange.bound(4, 15), count: 3},
    'Get bound range (generated) with maxCount');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {query: 'Doesn\'t exist'}, 'Non existent key');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {count: 0}, 'zero maxCount');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {count: 4294967295}, 'Max value count');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {query: IDBKeyRange.upperBound('0')},
    'Query with empty range where  first key < upperBound');

object_store_get_all_keys_test(
    /*storeName=*/ 'out-of-line',
    /*options=*/ {query: IDBKeyRange.lowerBound('zz')},
    'Query with empty range where lowerBound < last key');
