// META: title=test WebNN API buffer operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlbuffer

if (navigator.ml) {
  testCreateWebNNBuffer('create', 4);
  testDestroyWebNNBuffer('destroyTwice');
  testReadWebNNBuffer('read');
  testWriteWebNNBuffer('write');
  testDispatchWebNNBuffer('dispatch');
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
