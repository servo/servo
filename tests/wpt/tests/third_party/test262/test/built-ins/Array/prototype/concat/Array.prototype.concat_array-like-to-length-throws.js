// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat array like to length throws
features: [Symbol.isConcatSpreadable]
---*/
var spreadableWithBrokenLength = {
  "length": {
    valueOf: null,
    toString: null
  },
  "1": "A",
  "3": "B",
  "5": "C"
};
spreadableWithBrokenLength[Symbol.isConcatSpreadable] = true;
var obj2 = {
  length: 3,
  "0": "0",
  "1": "1",
  "2": "2"
};
var arr = ["X", "Y", "Z"];
assert.throws(TypeError, function() {
  Array.prototype.concat.call(spreadableWithBrokenLength, obj2, arr);
}, 'Array.prototype.concat.call(spreadableWithBrokenLength, obj2, arr) throws a TypeError exception');
