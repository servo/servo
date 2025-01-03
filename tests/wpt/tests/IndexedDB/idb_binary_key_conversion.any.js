// META: title=Verify the conversion of various types of BufferSource
// META: global=window,worker

// Spec: http://w3c.github.io/IndexedDB/#key-construct

'use strict';

test(function() {
  let binary = new ArrayBuffer(0);
  let key = IDBKeyRange.lowerBound(binary).lower;

  assert_true(key instanceof ArrayBuffer);
  assert_equals(key.byteLength, 0);
  assert_equals(key.byteLength, binary.byteLength);
}, 'Empty ArrayBuffer');

test(function() {
  let binary = new ArrayBuffer(4);
  let dataView = new DataView(binary);
  dataView.setUint32(0, 1234567890);

  let key = IDBKeyRange.lowerBound(binary).lower;

  assert_true(key instanceof ArrayBuffer);
  assert_equals(key.byteLength, 4);
  assert_equals(dataView.getUint32(0), new DataView(key).getUint32(0));
}, 'ArrayBuffer');

test(function() {
  let binary = new ArrayBuffer(4);
  let dataView = new DataView(binary);
  dataView.setUint32(0, 1234567890);

  let key = IDBKeyRange.lowerBound(dataView).lower;

  assert_true(key instanceof ArrayBuffer);
  assert_equals(key.byteLength, 4);
  assert_equals(dataView.getUint32(0), new DataView(key).getUint32(0));
}, 'DataView');

test(function() {
  let binary = new ArrayBuffer(4);
  let dataView = new DataView(binary);
  let int8Array = new Int8Array(binary);
  int8Array.set([16, -32, 64, -128]);

  let key = IDBKeyRange.lowerBound(int8Array).lower;
  let keyInInt8Array = new Int8Array(key);

  assert_true(key instanceof ArrayBuffer);
  assert_equals(key.byteLength, 4);
  for (let i = 0; i < int8Array.length; i++) {
    assert_equals(keyInInt8Array[i], int8Array[i]);
  }
}, 'TypedArray(Int8Array)');

test(function() {
  let binary = new ArrayBuffer(4);
  let dataView = new DataView(binary);
  let int8Array = new Int8Array(binary);
  int8Array.set([16, -32, 64, -128]);

  let key = IDBKeyRange.lowerBound([int8Array]).lower;

  assert_true(key instanceof Array);
  assert_true(key[0] instanceof ArrayBuffer);
  assert_equals(key[0].byteLength, 4);

  let keyInInt8Array = new Int8Array(key[0]);

  for (let i = 0; i < int8Array.length; i++) {
    assert_equals(keyInInt8Array[i], int8Array[i]);
  }
}, 'Array of TypedArray(Int8Array)');
