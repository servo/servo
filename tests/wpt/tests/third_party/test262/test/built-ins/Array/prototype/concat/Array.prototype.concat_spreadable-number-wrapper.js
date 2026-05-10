// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat Symbol.isConcatSpreadable number wrapper
includes: [compareArray.js]
features: [Symbol.isConcatSpreadable]
---*/
var num = new Number(true)
// Number wrapper objects are not concat-spreadable by default
assert.compareArray([num], [].concat(num), '[num] must return the same value returned by [].concat(num)');

// Number wrapper objects may be individually concat-spreadable
num[Symbol.isConcatSpreadable] = true;
num.length = 3;
num[0] = 1, num[1] = 2, num[2] = 3;
assert.compareArray([1, 2, 3], [].concat(num), '[1, 2, 3] must return the same value returned by [].concat(num)');

Number.prototype[Symbol.isConcatSpreadable] = true;
// Number wrapper objects may be concat-spreadable
assert.compareArray([], [].concat(new Number(123)),
  '[] must return the same value returned by [].concat(new Number(123))'
);
Number.prototype[0] = 1;
Number.prototype[1] = 2;
Number.prototype[2] = 3;
Number.prototype.length = 3;
assert.compareArray([1, 2, 3], [].concat(new Number(123)),
  '[1, 2, 3] must return the same value returned by [].concat(new Number(123))'
);

// Number values are never concat-spreadable
assert.compareArray([true], [].concat(true), '[true] must return the same value returned by [].concat(true)');
delete Number.prototype[Symbol.isConcatSpreadable];
delete Number.prototype[0];
delete Number.prototype[1];
delete Number.prototype[2];
delete Number.prototype.length;
