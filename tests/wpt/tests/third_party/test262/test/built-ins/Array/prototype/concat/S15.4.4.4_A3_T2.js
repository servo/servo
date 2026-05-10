// Copyright (c) 2014 Ecma International.  All rights reserved.
// See LICENSE or https://github.com/tc39/test262/blob/HEAD/LICENSE

/*---
esid: sec-array.prototype.concat
info: Array.prototype.concat uses [[Get]] on 'length' to determine array length
es5id: 15.4.4.4_A3_T2
description: >
  checking whether non-ownProperties are seen, copied by Array.prototype.concat: Array.prototype[1]
---*/

var a = [0];

assert.sameValue(a.length, 1, 'The value of a.length is expected to be 1');

a.length = 3;

assert.sameValue(a[1], undefined, 'The value of a[1] is expected to equal undefined');
assert.sameValue(a[2], undefined, 'The value of a[2] is expected to equal undefined');

Array.prototype[2] = 2;

assert.sameValue(a[1], undefined, 'The value of a[1] is expected to equal undefined');
assert.sameValue(a[2], 2, 'The value of a[2] is expected to be 2');
assert.sameValue(a.hasOwnProperty('1'), false, 'a.hasOwnProperty("1") must return false');
assert.sameValue(a.hasOwnProperty('2'), false, 'a.hasOwnProperty("2") must return false');

var b = a.concat();

assert.sameValue(b.length, 3, 'The value of b.length is expected to be 3');
assert.sameValue(b[0], 0, 'The value of b[0] is expected to be 0');
assert.sameValue(b[1], undefined, 'The value of b[1] is expected to equal undefined');
assert.sameValue(b[2], 2, 'The value of b[2] is expected to be 2');
assert.sameValue(b.hasOwnProperty('1'), false, 'b.hasOwnProperty("1") must return false');
assert.sameValue(b.hasOwnProperty('2'), true, 'b.hasOwnProperty("2") must return true');
