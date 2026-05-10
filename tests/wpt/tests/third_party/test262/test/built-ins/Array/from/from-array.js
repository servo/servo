// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Passing a valid array
esid: sec-array.from
---*/

var array = [0, 'foo', , Infinity];
var result = Array.from(array);

assert.sameValue(result.length, 4, 'The value of result.length is expected to be 4');
assert.sameValue(result[0], 0, 'The value of result[0] is expected to be 0');
assert.sameValue(result[1], 'foo', 'The value of result[1] is expected to be "foo"');
assert.sameValue(result[2], undefined, 'The value of result[2] is expected to equal undefined');
assert.sameValue(result[3], Infinity, 'The value of result[3] is expected to equal Infinity');

assert.notSameValue(
  result, array,
  'The value of result is expected to not equal the value of `array`'
);

assert(result instanceof Array, 'The result of evaluating (result instanceof Array) is expected to be true');
