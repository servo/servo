// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: RegExp used when `this` value does not define a constructor
info: |
    [...]
    5. Let C be SpeciesConstructor(rx, %RegExp%).
    [...]

    ES6 Section 7.3.20 SpeciesConstructor ( O, defaultConstructor )

    1. Assert: Type(O) is Object.
    2. Let C be Get(O, "constructor").
    3. ReturnIfAbrupt(C).
    4. If C is undefined, return defaultConstructor.
features: [Symbol.split]
---*/

var re = /[db]/;
var result;
re.constructor = undefined;

result = re[Symbol.split]('abcde');

assert(Array.isArray(result));
assert.sameValue(result.length, 3);
assert.sameValue(result[0], 'a');
assert.sameValue(result[1], 'c');
assert.sameValue(result[2], 'e');
