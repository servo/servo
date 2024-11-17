// META: script=nested-cloning-common.js
// META: script=support.js
// META: script=support-promises.js

'use strict';

// Define constants used to populate object stores and indexes.
const alphabet = 'abcdefghijklmnopqrstuvwxyz'.split('');
const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');
const vowels = 'aeiou'.split('');

// Setup multiple object stores to test `getAllKeys()`, `getAll()` and
// `getAllRecords()`.
function object_store_get_all_test(func, name) {
  indexeddb_test((test, connection, transaction) => {
    // Record the keys and values added to each object store during test setup.
    // Maps each object store's name to an array of records.  Tests can use
    // these records to verify the actual results from a get all request.
    self.expectedObjectStoreRecords = {
      'generated': [],
      'out-of-line': [],
      'empty': [],
      'large-values': [],
    };

    // Create an object store with auto-generated, auto-incrementing, inline
    // keys.
    let store = connection.createObjectStore(
        'generated', {autoIncrement: true, keyPath: 'id'});
    alphabet.forEach(letter => {
      store.put({ch: letter});

      const generatedKey = alphabet.indexOf(letter) + 1;
      expectedObjectStoreRecords['generated'].push(
          {key: generatedKey, primaryKey: generatedKey, value: {ch: letter}});
    });

    // Create an object store with out-of-line keys.
    store = connection.createObjectStore('out-of-line');
    alphabet.forEach(letter => {
      store.put(`value-${letter}`, letter);

      expectedObjectStoreRecords['out-of-line'].push(
          {key: letter, primaryKey: letter, value: `value-${letter}`});
    });

    // Create an empty object store.
    store = connection.createObjectStore('empty');

    // Create an object store with 3 large values.
    // `largeValue()` generates the value using the key as the seed.
    // The keys start at 0 and then increment by 1.
    store = connection.createObjectStore('large-values');
    for (let i = 0; i < 3; i++) {
      const value = largeValue(/*size=*/ wrapThreshold, /*seed=*/ i);
      store.put(value, i);

      expectedObjectStoreRecords['large-values'].push(
          {key: i, primaryKey: i, value});
    }
  }, func, name);
}

// Test `getAllRecords()` on `storeName` with the given `options`.
//  - `options` is an `IDBGetAllRecordsOptions ` dictionary that may contain a
//    `query`, `direction` and `count`.
function object_store_get_all_records_test(storeName, options, description) {
  object_store_get_all_test((test, connection) => {
    const request =
        createGetAllRecordsRequest(test, connection, storeName, options);
    request.onsuccess = test.step_func(event => {
      // Build the expected results.
      let expectedResults = expectedObjectStoreRecords[storeName];
      expectedResults = applyGetAllRecordsOptions(expectedResults, options);

      const actualResults = event.target.result;
      assert_records_equals(actualResults, expectedResults);
      test.done();
    });
  }, description);
}

// Setup multiple object stores with indexes to test `getAllKeys()`, `getAll()`
// and `getAllRecords()`.
function index_get_all_test(func, name) {
  indexeddb_test(function(test, connection, transaction) {
    // Record the keys and values added to each object store's index during test
    // setup. Maps each object store's name to the array of records in its
    // index. Tests can use these records to verify the actual results from a
    // get all request.
    self.expectedIndexRecords = {
      'generated': [],
      'out-of-line': [],
      'out-of-line-not-unique': [],
      'out-of-line-multi': [],
      'empty': [],
      'large-values': [],
    };

    // Create an object store with auto-incrementing, inline keys.  Create an
    // index on the uppercase letter property `upper`.
    let store = connection.createObjectStore(
        'generated', {autoIncrement: true, keyPath: 'id'});
    let index = store.createIndex('test_idx', 'upper');
    alphabet.forEach(function(letter) {
      const value = {ch: letter, upper: letter.toUpperCase()};
      store.put(value);

      const generatedKey = alphabet.indexOf(letter) + 1;
      expectedIndexRecords['generated'].push(
          {key: value.upper, primaryKey: generatedKey, value});
    });

    // Create an object store with out-of-line keys.  Create an index on the
    // uppercase letter property `upper`.
    store = connection.createObjectStore('out-of-line', null);
    index = store.createIndex('test_idx', 'upper');
    alphabet.forEach(function(letter) {
      const value = {ch: letter, upper: letter.toUpperCase()};
      store.put(value, letter);

      expectedIndexRecords['out-of-line'].push(
          {key: value.upper, primaryKey: letter, value});
    });

    // Create an index on the `half` property, which is not unique, with two
    // possible values: `first` and `second`.
    store = connection.createObjectStore('out-of-line-not-unique', null);
    index = store.createIndex('test_idx', 'half');
    alphabet.forEach(function(letter) {
      let half = 'first';
      if (letter > 'm') {
        half = 'second';
      }

      const value = {ch: letter, half};
      store.put(value, letter);

      expectedIndexRecords['out-of-line-not-unique'].push(
          {key: half, primaryKey: letter, value});
    });

    // Create a multi-entry index on `attribs`, which is an array of strings.
    store = connection.createObjectStore('out-of-line-multi', null);
    index = store.createIndex('test_idx', 'attribs', {multiEntry: true});
    alphabet.forEach(function(letter) {
      let attrs = [];
      if (['a', 'e', 'i', 'o', 'u'].indexOf(letter) != -1) {
        attrs.push('vowel');
      } else {
        attrs.push('consonant');
      }
      if (letter == 'a') {
        attrs.push('first');
      }
      if (letter == 'z') {
        attrs.push('last');
      }
      const value = {ch: letter, attribs: attrs};
      store.put(value, letter);

      for (let attr of attrs) {
        expectedIndexRecords['out-of-line-multi'].push(
            {key: attr, primaryKey: letter, value});
      }
    });

    // Create an empty index.
    store = connection.createObjectStore('empty', null);
    index = store.createIndex('test_idx', 'upper');

    // Create an object store and index with 3 large values and their seed.  Use
    // the large value's seed as the index key.
    store = connection.createObjectStore('large-values');
    index = store.createIndex('test_idx', 'seed');
    for (let i = 0; i < 3; i++) {
      const seed = i;
      const randomValue = largeValue(/*size=*/ wrapThreshold, seed);
      const recordValue = {seed, randomValue};
      store.put(recordValue, i);

      expectedIndexRecords['large-values'].push(
          {key: seed, primaryKey: i, value: recordValue});
    }
  }, func, name);
}

function index_get_all_records_test(storeName, options, description) {
  index_get_all_test((test, connection) => {
    const request = createGetAllRecordsRequest(
        test, connection, storeName, options, 'test_idx');
    request.onsuccess = test.step_func(event => {
      // Build the expected results.
      let expectedResults = expectedIndexRecords[storeName];
      expectedResults = applyGetAllRecordsOptions(expectedResults, options);

      const actualResults = event.target.result;
      assert_records_equals(actualResults, expectedResults);
      test.done();
    });
  }, description);
}

// Create a `getAllRecords()` request for either `storeName` or
// `optionalIndexName`.
//  - `options` is an `IDBGetAllRecordsOptions ` dictionary that may contain a
//    `query`, `direction` and `count`.
function createGetAllRecordsRequest(
    test, connection, storeName, options, optionalIndexName) {
  const transaction = connection.transaction(storeName, 'readonly');
  let queryTarget = transaction.objectStore(storeName);
  if (optionalIndexName) {
    queryTarget = queryTarget.index(optionalIndexName);
  }
  const request = queryTarget.getAllRecords(options);
  request.onerror = test.unreached_func('getAllRecords request must succeed');
  return request;
}

// Returns the array of `records` that satisfy `options`.  Tests may use this to
// generate expected results.
//  - `records` is an array of objects where each object has the properties:
//    `key`, `primaryKey`, and `value`.
//  - `options` is an `IDBGetAllRecordsOptions ` dictionary that may contain a
//    `query`, `direction` and `count`.
function applyGetAllRecordsOptions(records, options) {
  if (!options) {
    return records;
  }

  // Remove records that don't satisfy the query.
  if (options.query) {
    let query = options.query;
    if (!(query instanceof IDBKeyRange)) {
      // Create an IDBKeyRange for the query's key value.
      query = IDBKeyRange.only(query);
    }
    records = records.filter(record => query.includes(record.key));
  }

  // Remove duplicate records.
  if (options.direction === 'nextunique' ||
      options.direction === 'prevunique') {
    const uniqueRecords = [];
    records.forEach(record => {
      if (!uniqueRecords.some(
              unique => IDBKeyRange.only(unique.key).includes(record.key))) {
        uniqueRecords.push(record);
      }
    });
    records = uniqueRecords;
  }

  // Reverse the order of the records.
  if (options.direction === 'prev' || options.direction === 'prevunique') {
    records = records.slice().reverse();
  }

  // Limit the number of records.
  if (options.count) {
    records = records.slice(0, options.count);
  }
  return records;
}

function isArrayOrArrayBufferView(value) {
  return Array.isArray(value) || ArrayBuffer.isView(value);
}

// This function compares the string representation of the arrays because
// `assert_array_equals()` is too slow for large values.
function assert_large_array_equals(actual, expected, description) {
  const array_string = actual.join(',');
  const expected_string = expected.join(',');
  assert_equals(array_string, expected_string, description);
}

// Verifies each record from the results of `getAllRecords()`.
function assert_record_equals(actual_record, expected_record) {
  assert_class_string(
      actual_record, 'IDBRecord', 'The record must be an IDBRecord');
  assert_idl_attribute(
      actual_record, 'key', 'The record must have a key attribute');
  assert_idl_attribute(
      actual_record, 'primaryKey',
      'The record must have a primaryKey attribute');
  assert_idl_attribute(
      actual_record, 'value', 'The record must have a value attribute');

  // Verify the key properties.
  assert_equals(
      actual_record.primaryKey, expected_record.primaryKey,
      'The record must have the expected primaryKey');
  assert_equals(
      actual_record.key, expected_record.key,
      'The record must have the expected key');

  // Verify the value.
  const actual_value = actual_record.value;
  const expected_value = expected_record.value;
  if (isArrayOrArrayBufferView(expected_value)) {
    assert_large_array_equals(
        actual_value, expected_value,
        'The record must have the expected value');
  } else if (typeof expected_value === 'object') {
    // Verify each property of the object value.
    for (let property_name of Object.keys(expected_value)) {
      if (isArrayOrArrayBufferView(expected_value[property_name])) {
        // Verify the array property value.
        assert_large_array_equals(
            actual_value[property_name], expected_value[property_name],
            `The record must contain the array value "${
                JSON.stringify(
                    expected_value)}" with property "${property_name}"`);
      } else {
        // Verify the primitive property value.
        assert_equals(
            actual_value[property_name], expected_value[property_name],
            `The record must contain the value "${
                JSON.stringify(
                    expected_value)}" with property "${property_name}"`);
      }
    }
  } else {
    // Verify the primitive value.
    assert_equals(
        actual_value, expected_value,
        'The record must have the expected value');
  }
}

// Verifies the results from `getAllRecords()`, which is an array of records:
// [
//   { 'key': key1, 'primaryKey': primary_key1, 'value': value1 },
//   { 'key': key2, 'primaryKey': primary_key2, 'value': value2 },
//   ...
// ]
function assert_records_equals(actual_records, expected_records) {
  assert_true(
      Array.isArray(actual_records),
      'The records must be an array of IDBRecords');
  assert_equals(
      actual_records.length, expected_records.length,
      'The records array must contain the expected number of records');

  for (let i = 0; i < actual_records.length; i++) {
    assert_record_equals(actual_records[i], expected_records[i]);
  }
}
