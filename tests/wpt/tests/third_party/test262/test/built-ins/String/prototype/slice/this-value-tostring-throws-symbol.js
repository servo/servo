// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.slice
description: If this value is a Symbol, String.prototype.slice should throw a TypeError
info: |
  String.prototype.slice ( _start_, _end_ )
  
  1. Let O be ? RequireObjectCoercible(this value).
  2. Let _S_ be ? ToString(_O_).

  ToString (_argument_)
  
  2. If _argument_ is a Symbol, throw a *TypeError* exception.
features: [Symbol]
---*/
assert.throws(TypeError, function () {
    String.prototype.slice.call(Symbol())
}, "If this value is a Symbol, String.prototype.slice should throw a TypeError")
