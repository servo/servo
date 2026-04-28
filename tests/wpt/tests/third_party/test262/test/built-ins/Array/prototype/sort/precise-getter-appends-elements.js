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
    array.push('foo');
    array.push('bar');
    return this.foo;
  },
  set(v) {
    this.foo = v;
  }
});

array.sort();

assert.sameValue(array[0], 'a');
assert.sameValue(array[1], 'b');
assert.sameValue('2' in array, true);
assert.sameValue(array.hasOwnProperty('2'), true);
assert.sameValue(array[3], 'd');
assert.sameValue(array[4], undefined);
assert.sameValue(array[5], undefined);
assert.sameValue(array[6], undefined);
assert.sameValue('7' in array, false);
assert.sameValue(array.hasOwnProperty('7'), false);
assert.sameValue(array[8], 'foo');
assert.sameValue(array[9], 'bar');
assert.sameValue(array.length, 10);
assert.sameValue(array.foo, 'c');

assert.sameValue(array[2], 'c');
assert.sameValue(array[10], 'foo');
assert.sameValue(array[11], 'bar');
assert.sameValue(array.length, 12);
assert.sameValue(array.foo, 'c');
