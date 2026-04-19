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

const array = [undefined, 'c', /*hole*/, 'b', undefined, /*hole*/, 'a', 'd'];

Object.defineProperty(array, '2', {
  get() {
    array.length = array.length - 2;
    return this.foo;
  },
  set(v) {
    this.foo = v;
  }
});

array.sort();

assert.sameValue(array[0], 'b');
assert.sameValue(array[1], 'c');
assert.sameValue(array[3], undefined);
assert.sameValue(array[4], undefined);
assert.sameValue('5' in array, false);
assert.sameValue(array.hasOwnProperty('5'), false);
assert.sameValue(array.length, 6);
assert.sameValue(array.foo, undefined);

assert.sameValue(array[2], undefined);
assert.sameValue(array.length, 4);
