// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: array-concat-non-array
includes: [compareArray.js]
---*/

//
// test262 --command "v8 --harmony-classes  --harmony_object_literals" array-concat-non-array
//
class NonArray {
  constructor() {
    Array.apply(this, arguments);
  }
}

var obj = new NonArray(1, 2, 3);
var result = Array.prototype.concat.call(obj, 4, 5, 6);
assert.sameValue(Array, result.constructor, 'The value of Array is expected to equal the value of result.constructor');
assert.sameValue(
  result instanceof NonArray,
  false,
  'The result of evaluating (result instanceof NonArray) is expected to be false'
);
assert.compareArray(result, [obj, 4, 5, 6], 'The value of result is expected to be [obj, 4, 5, 6]');
