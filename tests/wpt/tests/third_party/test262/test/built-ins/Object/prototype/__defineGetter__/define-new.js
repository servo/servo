// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-additional-properties-of-the-object.prototype-object
description: Behavior when property does not exist
info: |
    [...]
    3. Let desc be PropertyDescriptor{[[Get]]: getter, [[Enumerable]]: true,
       [[Configurable]]: true}.
    4. Let key be ? ToPropertyKey(P).
    5. Perform ? DefinePropertyOrThrow(O, key, desc).
    6. Return undefined.
includes: [propertyHelper.js]
features: [__getter__]
---*/

var subject = {};
var get = function() {};
var desc, result;

result = subject.__defineGetter__('stringAcsr', get);

desc = Object.getOwnPropertyDescriptor(subject, 'stringAcsr');

verifyProperty(subject, "stringAcsr", {
  enumerable: true,
  configurable: true
});

assert.sameValue(desc.get, get, 'descriptor `get` method');
assert.sameValue(desc.set, undefined, 'descriptor `set` method');
assert.sameValue(desc.value, undefined, 'descriptor `value` property');

assert.sameValue(result, undefined, 'method return value');
