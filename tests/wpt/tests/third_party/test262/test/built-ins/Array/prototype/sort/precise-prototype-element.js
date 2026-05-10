// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.sort
description: >
  Previously implementation-defined aspects of Array.prototype.sort.
info: |
  Historically, many aspects of Array.prototype.sort remained
  implementation-defined. https://github.com/tc39/ecma262/pull/1585
  described some behaviors more precisely, reducing the amount of cases
  that result in an implementation-defined sort order.
---*/

Object.prototype[2] = 4;
const array = [undefined, 3, /*hole*/, 2, undefined, /*hole*/, 1];
array.sort();

assert.sameValue(array[0], 1);
assert.sameValue(array[1], 2);
assert.sameValue(array[2], 3);
assert.sameValue(array[3], 4);
assert.sameValue(array[4], undefined);
assert.sameValue(array[5], undefined);
assert.sameValue('6' in array, false);
assert.sameValue(array.hasOwnProperty('6'), false);
assert.sameValue(array.length, 7);
