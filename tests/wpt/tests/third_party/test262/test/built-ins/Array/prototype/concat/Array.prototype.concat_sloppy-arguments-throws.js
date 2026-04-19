// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat sloppy arguments throws
features: [Symbol.isConcatSpreadable]
---*/
var args = (function(a) {
  return arguments;
})(1, 2, 3);
Object.defineProperty(args, 0, {
  get: function() {
    throw new Test262Error();
  }
});
args[Symbol.isConcatSpreadable] = true;
assert.throws(Test262Error, function() {
  return [].concat(args, args);
}, 'return [].concat(args, args) throws a Test262Error exception');
