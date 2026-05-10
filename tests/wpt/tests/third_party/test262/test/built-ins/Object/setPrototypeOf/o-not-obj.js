// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.18
description: Object.setPrototypeOf invoked with a non-object value
info: |
    1. Let O be RequireObjectCoercible(O).
    2. ReturnIfAbrupt(O).
    3. If Type(proto) is neither Object nor Null, throw a TypeError exception.
    4. If Type(O) is not Object, return O.
features: [Symbol]
---*/

var symbol;

assert.sameValue(Object.setPrototypeOf(true, null), true);
assert.sameValue(Object.setPrototypeOf(3, null), 3);
assert.sameValue(Object.setPrototypeOf('string', null), 'string');

symbol = Symbol('s');
assert.sameValue(Object.setPrototypeOf(symbol, null), symbol);
