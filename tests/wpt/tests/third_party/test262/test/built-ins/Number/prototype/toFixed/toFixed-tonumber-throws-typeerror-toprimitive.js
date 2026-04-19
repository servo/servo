// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.tofixed
description: >
    ToNumber called by Number.prototype.toFixed throws when ToPrimitive Throws
info: |
    Number.prototype.toFixed ( _fractionDigits_ )

    2. Let _f_ be ? ToIntegerOrInfinity(_fractionDigits_).

    ToIntegerOrInfinity (_argument_)

    1. Let _number_ be ? ToNumber(_argument_).

    ToNumber (_argument_)

    7. Assert: _argument_ is an Object.
    8. Let _primValue_ be ? ToPrimitive(_argument_, ~number~).
---*/
assert.throws(TypeError, function() {
    Number.prototype.toFixed.call(0, {valueOf: undefined, toString: undefined})
}, "TypeError is thrown when _fractionDigits_ cannot be converted to a primitive")
