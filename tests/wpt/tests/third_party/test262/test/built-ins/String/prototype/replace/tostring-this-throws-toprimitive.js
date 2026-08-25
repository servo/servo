// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.replace
description: >
    String.prototype.replace throws a TypeError when this value cannot be converted to a primitive.
info: |
    String.prototype.replace ( _searchValue_, _replaceValue_ )

    1. Let _O_ be ? RequireObjectCoercible(*this* value).
    3. Let _string_ be ? ToString(_O_).

    ToString (_argument_)

    10. Let _primValue_ be ? ToPrimitive(_argument_, ~string~).
---*/
assert.throws(TypeError, function () {
    String.prototype.replace.call({toString: undefined, valueOf: undefined})
}, "String.prototype.replace should throw a TypeError in its ToPrimitive step.")
