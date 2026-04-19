// Copyright (C) 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: Non-string values of `@@toStringTag` property are ignored
info: |
    14. Else, let builtinTag be "Object".
    15. Let tag be ? Get(O, @@toStringTag).
    16. If Type(tag) is not String, let tag be builtinTag.
    17. Return the string-concatenation of "[object ", tag, and "]".
features: [BigInt, Symbol.toStringTag]
---*/

let custom1 = BigInt(0);
let custom2 = Object(BigInt(0));

Object.defineProperty(BigInt.prototype, Symbol.toStringTag, {value: undefined});
assert.sameValue(Object.prototype.toString.call(custom1), '[object Object]');
assert.sameValue(Object.prototype.toString.call(custom2), '[object Object]');

Object.defineProperty(BigInt.prototype, Symbol.toStringTag, {value: null});
assert.sameValue(Object.prototype.toString.call(custom1), '[object Object]');
assert.sameValue(Object.prototype.toString.call(custom2), '[object Object]');

Object.defineProperty(BigInt.prototype, Symbol.toStringTag, {value: Symbol.toStringTag});
assert.sameValue(Object.prototype.toString.call(custom1), '[object Object]');
assert.sameValue(Object.prototype.toString.call(custom2), '[object Object]');

Object.defineProperty(BigInt.prototype, Symbol.toStringTag, {value: 86});
assert.sameValue(Object.prototype.toString.call(custom1), '[object Object]');
assert.sameValue(Object.prototype.toString.call(custom2), '[object Object]');

Object.defineProperty(BigInt.prototype, Symbol.toStringTag, {value: new String('test262')});
assert.sameValue(Object.prototype.toString.call(custom1), '[object Object]');
assert.sameValue(Object.prototype.toString.call(custom2), '[object Object]');

Object.defineProperty(BigInt.prototype, Symbol.toStringTag, {value: {}});
assert.sameValue(Object.prototype.toString.call(custom1), '[object Object]');
assert.sameValue(Object.prototype.toString.call(custom2), '[object Object]');

Object.defineProperty(BigInt.prototype, Symbol.toStringTag, {value: _ => 'str'});
assert.sameValue(Object.prototype.toString.call(custom1), '[object Object]');
assert.sameValue(Object.prototype.toString.call(custom2), '[object Object]');
