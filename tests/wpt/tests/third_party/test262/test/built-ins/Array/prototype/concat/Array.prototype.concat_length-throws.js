// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat length throws
features: [Symbol.isConcatSpreadable]
---*/
var spreadablePoisonedLengthGetter = {};
spreadablePoisonedLengthGetter[Symbol.isConcatSpreadable] = true;
Object.defineProperty(spreadablePoisonedLengthGetter, "length", {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  [].concat(spreadablePoisonedLengthGetter);
}, '[].concat(spreadablePoisonedLengthGetter) throws a Test262Error exception');

assert.throws(Test262Error, function() {
  Array.prototype.concat.call(spreadablePoisonedLengthGetter, 1, 2, 3);
}, 'Array.prototype.concat.call(spreadablePoisonedLengthGetter, 1, 2, 3) throws a Test262Error exception');
