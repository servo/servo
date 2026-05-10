// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.replace
description: >
    If this value is a Symbol String.prototype.replace should throw a TypeError.
info: |
    String.prototype.replace ( _searchValue_, _replaceValue_ )

    1. Let _O_ be ? RequireObjectCoercible(*this* value).
    3. Let _string_ be ? ToString(_O_).
---*/
assert.throws(TypeError, function () {
    String.prototype.replace.call(Symbol())
}, "If this value is a Symbol String.prototype.replace should throw a TypeError.")
