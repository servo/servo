// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var BUGNUMBER = 1352429;
var summary = 'Error message should provide enough infomation for use of in operator';

// These test cases check if long string is omitted properly.
assert.throws(TypeError, () => 'subString' in 'base');
assert.throws(TypeError, () => 'this is subString' in 'base');
assert.throws(TypeError, () => 'subString' in 'this is baseString');
assert.throws(TypeError, () => 'this is subString' in 'this is base');
assert.throws(TypeError, () => 'HEAD' + 'subString'.repeat(30000) in 'HEAD' + 'base'.repeat(30000));

// These test cases check if it does not crash and throws appropriate error.
assert.throws(TypeError, () => { 1 in 'hello' });
assert.throws(TypeError, () => { 'hello' in 1 });
assert.throws(TypeError, () => { 'hello' in null });
assert.throws(TypeError, () => { null in 'hello' });
assert.throws(TypeError, () => { null in null });
assert.throws(TypeError, () => { 'hello' in true });
assert.throws(TypeError, () => { false in 1.1 });
assert.throws(TypeError, () => { Symbol.iterator in undefined });
assert.throws(TypeError, () => { [] in undefined });
assert.throws(TypeError, () => { /a/ in 'hello' });
var str = 'hello';
assert.throws(TypeError, () => { str in 'hello' });
class A {};
assert.throws(TypeError, () => { new A() in undefined });
var a = new A();
a.b = 1.1;
assert.throws(TypeError, () => { a.b in 1.1 });

