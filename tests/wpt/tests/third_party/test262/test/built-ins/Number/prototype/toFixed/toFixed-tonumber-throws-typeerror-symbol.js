// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.tofixed
description: >
    Number.prototype.toFixed(_fractionDigits_) must throw a TypeError when _fractionDigits_ is a Symbol
info: |
    Number.prototype.toFixed ( _fractionDigits_ )

    2. Let _f_ be ? ToIntegerOrInfinity(_fractionDigits_).

    ToIntegerOrInfinity (_argument_)

    1. Let _number_ be ? ToNumber(_argument_).

    ToNumber (_argument_)

    2. If _argument_ is either a Symbol or a BigInt, throw a *TypeError* exception.
features: [Symbol]
---*/
assert.throws(TypeError, function() {
    (0).toFixed(Symbol())
}, "Number.prototype.toFixed must throw a TypeError if _fractionDigits_ is Symbol")
