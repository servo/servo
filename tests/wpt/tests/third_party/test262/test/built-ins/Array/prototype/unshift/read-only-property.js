// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.unshift
description: >
    Array.prototype.unshift throws TypeError when a property is read-only.
info: |
    Array.prototype.unshift ( ..._items_ )

    1. Let _O_ be ? ToObject(*this* value).
    2. Let _len_ be ? LengthOfArrayLike(_O_).
    3. Let _argCount_ be the number of elements in _items_.
    4. If _argCount_ > 0, then
        a. If _len_ + _argCount_ > 2<sup>53</sup> - 1, throw a *TypeError* exception.
        b. Let _k_ be _len_.
        c. Repeat, while _k_ > 0,
        d. Let _j_ be *+0*<sub>𝔽</sub>.
        e. For each element _E_ of _items_, do
            i. Perform ? Set(_O_, ! ToString(_j_), _E_, *true*).
---*/
assert.throws(TypeError, function () {
    Array.prototype.unshift.call({get 0 () {}}, 0)
}, "Array.prototype.unshift throws TypeError when a property is read-only.")
