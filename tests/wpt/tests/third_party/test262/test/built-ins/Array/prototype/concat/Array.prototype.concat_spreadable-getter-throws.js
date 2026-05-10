// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat Symbol.isConcatSpreadable getter throws
features: [Symbol.isConcatSpreadable]
---*/
var spreadablePoisonedGetter = {};
Object.defineProperty(spreadablePoisonedGetter, Symbol.isConcatSpreadable, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  [].concat(spreadablePoisonedGetter);
}, '[].concat(spreadablePoisonedGetter) throws a Test262Error exception');

assert.throws(Test262Error, function() {
  Array.prototype.concat.call(spreadablePoisonedGetter, 1, 2, 3);
}, 'Array.prototype.concat.call(spreadablePoisonedGetter, 1, 2, 3) throws a Test262Error exception');
