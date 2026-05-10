// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Length coercion of `limit` argument
info: |
    [...]
    17. If limit is undefined, let lim be 2^32-1; else let lim be ? ToUint32(limit).
    [...]
features: [Symbol.split]
---*/

var result;

// ToUint32(-23) = 4294967273
result = /./[Symbol.split]('abc', -23);
assert(Array.isArray(result));
assert.sameValue(result.length, 4);

result = /./[Symbol.split]('abc', 1.9);
assert(Array.isArray(result));
assert.sameValue(result.length, 1);
assert.sameValue(result[0], '');

result = /./[Symbol.split]('abc', NaN);
assert(Array.isArray(result));
assert.sameValue(result.length, 0);
