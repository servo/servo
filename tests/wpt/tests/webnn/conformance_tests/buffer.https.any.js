// META: title=test WebNN API buffer operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlbuffer

if (navigator.ml) {
  testCreateWebNNBuffer('create', {dataType: 'float16', dimensions: [2, 3]});
  testCreateWebNNBuffer('create', {dataType: 'float32', dimensions: [1, 5]});
  testCreateWebNNBuffer('create', {dataType: 'int32', dimensions: [4]});
  testCreateWebNNBuffer('create', {dataType: 'uint8', dimensions: [3, 2, 4]});

  testCreateWebNNBufferFails(
      'createFailsEmptyDimension', {dataType: 'int32', dimensions: [2, 0, 3]});
  testCreateWebNNBufferFails('createFailsTooLarge', {
    dataType: 'int32',
    dimensions: [kMaxUnsignedLong, kMaxUnsignedLong, kMaxUnsignedLong]
  });

  testDestroyWebNNBuffer('destroyTwice');
  testReadWebNNBuffer('read');
  testWriteWebNNBuffer('write');
  testDispatchWebNNBuffer('dispatch');
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
