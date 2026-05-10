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

const logs = [];

Object.defineProperty(Object.prototype, '2', {
  get() {
    logs.push('get');
    return 4;
  },
  set(v) {
    logs.push(`set with ${v}`);
  }
});
const array = [undefined, 3, /*hole*/, 2, undefined, /*hole*/, 1];
let count = 0;
try {
  array.sort((a, b) => {
    if (++count === 3) {
      throw new Error('lolwat');
    }
    return b - a;
  });
} catch (exception) {
  logs.push(exception.message);
}

assert.sameValue(logs[0], 'get');
assert.sameValue(logs[1], 'lolwat');
assert.sameValue(logs.length, 2);

assert.sameValue(array[0], undefined);
assert.sameValue(array[1], 3);
assert.sameValue('2' in array, true);
assert.sameValue(array.hasOwnProperty('2'), false);
assert.sameValue(array[3], 2);
assert.sameValue(array[4], undefined);
assert.sameValue('5' in array, false);
assert.sameValue(array.hasOwnProperty('5'), false);
assert.sameValue(array[6], 1);
assert.sameValue(array.length, 7);
