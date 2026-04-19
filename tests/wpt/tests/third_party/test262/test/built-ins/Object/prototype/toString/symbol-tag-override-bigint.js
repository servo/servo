// Copyright (C) 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: String values of `@@toStringTag` property override built-in tags
info: |
    15. Let tag be ? Get(O, @@toStringTag).
    16. If Type(tag) is not String, let tag be builtinTag.
    17. Return the string-concatenation of "[object ", tag, and "]".
features: [BigInt, Symbol.toStringTag]
---*/

let custom1 = BigInt(0);
let custom2 = Object(BigInt(0));

Object.defineProperty(BigInt.prototype, Symbol.toStringTag, {value: 'test262'});
assert.sameValue(Object.prototype.toString.call(custom1), '[object test262]');
assert.sameValue(Object.prototype.toString.call(custom2), '[object test262]');
