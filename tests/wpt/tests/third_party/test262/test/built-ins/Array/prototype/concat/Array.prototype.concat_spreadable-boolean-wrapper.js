// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat Symbol.isConcatSpreadable boolean wrapper
includes: [compareArray.js]
features: [Symbol.isConcatSpreadable]
---*/
var bool = new Boolean(true)
// Boolean wrapper objects are not concat-spreadable by default
assert.compareArray([bool], [].concat(bool), '[bool] must return the same value returned by [].concat(bool)');

// Boolean wrapper objects may be individually concat-spreadable
bool[Symbol.isConcatSpreadable] = true;
bool.length = 3;
bool[0] = 1, bool[1] = 2, bool[2] = 3;
assert.compareArray([1, 2, 3], [].concat(bool),
  '[1, 2, 3] must return the same value returned by [].concat(bool)'
);

Boolean.prototype[Symbol.isConcatSpreadable] = true;
// Boolean wrapper objects may be concat-spreadable
assert.compareArray([], [].concat(new Boolean(true)),
  '[] must return the same value returned by [].concat(new Boolean(true))'
);
Boolean.prototype[0] = 1;
Boolean.prototype[1] = 2;
Boolean.prototype[2] = 3;
Boolean.prototype.length = 3;
assert.compareArray([1, 2, 3], [].concat(new Boolean(true)),
  '[1, 2, 3] must return the same value returned by [].concat(new Boolean(true))'
);

// Boolean values are never concat-spreadable
assert.compareArray([true], [].concat(true), '[true] must return the same value returned by [].concat(true)');
delete Boolean.prototype[Symbol.isConcatSpreadable];
delete Boolean.prototype[0];
delete Boolean.prototype[1];
delete Boolean.prototype[2];
delete Boolean.prototype.length;
