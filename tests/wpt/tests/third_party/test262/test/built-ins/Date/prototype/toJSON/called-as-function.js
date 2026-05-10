// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.tojson
description: >
    `this` value is resolved using strict mode semantics,
    throwing TypeError if called as top-level function.
info: |
    Date.prototype.toJSON ( key )

    1. Let O be ? ToObject(this value).

    ToObject ( argument )

    Argument Type: Undefined
    Result: Throw a TypeError exception.
features: [Symbol, Symbol.toPrimitive]
---*/

[Symbol.toPrimitive, "toString", "valueOf", "toISOString"].forEach(function(key) {
    Object.defineProperty(this, key, {
        get: function() {
            throw new Test262Error(String(key) + " lookup should not be performed");
        },
    });
}, this);

var toJSON = Date.prototype.toJSON;
assert.throws(TypeError, function() {
    toJSON();
});
