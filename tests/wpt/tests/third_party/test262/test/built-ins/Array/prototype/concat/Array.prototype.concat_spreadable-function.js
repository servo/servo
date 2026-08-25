// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat Symbol.isConcatSpreadable function
includes: [compareArray.js]
features: [Symbol.isConcatSpreadable]
---*/
var fn = function(a, b, c) {}
// Functions are not concat-spreadable by default
assert.compareArray([fn], [].concat(fn), '[fn] must return the same value returned by [].concat(fn)');

// Functions may be individually concat-spreadable
fn[Symbol.isConcatSpreadable] = true;
fn[0] = 1, fn[1] = 2, fn[2] = 3;
assert.compareArray([1, 2, 3], [].concat(fn), '[1, 2, 3] must return the same value returned by [].concat(fn)');

Function.prototype[Symbol.isConcatSpreadable] = true;
// Functions may be concat-spreadable
assert.compareArray([void 0, void 0, void 0], [].concat(function(a, b, c) {}),
  '[void 0, void 0, void 0] must return the same value returned by [].concat(function(a, b, c) {})'
);
Function.prototype[0] = 1;
Function.prototype[1] = 2;
Function.prototype[2] = 3;
assert.compareArray([1, 2, 3], [].concat(function(a, b, c) {}),
  '[1, 2, 3] must return the same value returned by [].concat(function(a, b, c) {})'
);

delete Function.prototype[Symbol.isConcatSpreadable];
delete Function.prototype[0];
delete Function.prototype[1];
delete Function.prototype[2];
