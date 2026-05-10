// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.substring
description: String.prototype.substring throws when this value cannot be converted to primitive.
info: |
  String.prototype.substring ( _start_, _end_ )
  
  1. Let O be ? RequireObjectCoercible(this value).
  2. Let _S_ be ? ToString(_O_).

  ToString (_argument_)
  
  10. Let _primValue_ be ? ToPrimitive(_argument_, ~string~).
---*/
assert.throws(TypeError, function () {
    String.prototype.substring.call({toString: undefined, valueOf: undefined})
}, "String.prototype.substring throws in its toprimitive step.")
