// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype-@@toprimitive
description: >
    `this` value is resolved using strict mode semantics,
    throwing TypeError if called as top-level function.
info: |
    Date.prototype [ @@toPrimitive ] ( hint )

    1. Let O be the this value.
    2. If Type(O) is not Object, throw a TypeError exception.

    ToObject ( argument )

    Argument Type: Undefined
    Result: Throw a TypeError exception.
features: [Symbol, Symbol.toPrimitive]
---*/

["toString", "valueOf"].forEach(function(key) {
    Object.defineProperty(this, key, {
        get: function() {
            throw new Test262Error(key + " lookup should not be performed");
        },
    });
}, this);

var toPrimitive = Date.prototype[Symbol.toPrimitive];
assert.throws(TypeError, function() {
    toPrimitive("default");
});
