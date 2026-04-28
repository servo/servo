// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Return abrupt from start as a Symbol.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  8. Let relativeStart be ToInteger(start).
  9. ReturnIfAbrupt(relativeStart).
  ...
features: [Symbol]
---*/

var s = Symbol(1);
assert.throws(TypeError, function() {
  [].copyWithin(0, s);
});
