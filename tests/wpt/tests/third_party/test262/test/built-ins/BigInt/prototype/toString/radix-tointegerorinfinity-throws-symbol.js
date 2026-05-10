// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tostring
description: > 
    If _radix_ is a BigInt or a Symbol, ToNumber called by BigInt.prototype.toString throws a TypeError.
info: |
    BigInt.prototype.toString ( [ _radix_ ] )

    3. Else, let _radixMV_ be ? ToIntegerOrInfinity(_radix_).

    ToIntegerOrInfinity (_argument_)

    1. Let _number_ be ? ToNumber(_argument_).

    ToNumber (_argument_)

    2. If _argument_ is either a Symbol or a BigInt, throw a *TypeError* exception.
features: [Symbol]
---*/
assert.throws(TypeError, function() {
    (0n).toString(Symbol());
}, "If _radix_ is Symbol, BigInt.prototype.toString must throw a TypeError")
