// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.iswellformed
description: >
  The method should coerce the receiver to a string.
info: |
  String.prototype.isWellFormed ( )

  2. Let S be ? ToString(O).
  …

features: [String.prototype.isWellFormed]
---*/

const tests = [
  [true, Boolean.prototype],
  [1, Number.prototype],
  [1n, BigInt.prototype],
];

for (const [v, proto] of tests) {
  proto.toString = function() {
    throw new Test262Error(`should not call toString on the prototype for ${typeof v}`);
  }
  let result = String.prototype.isWellFormed.call(v);
  delete proto.toString;
  assert.sameValue(result, true, `isWellFormed for ${typeof v}`);
}

Symbol.prototype.toString = function() { throw new TypeError("should not call toString on the prototype for Symbol"); }
assert.throws(TypeError, () => String.prototype.isWellFormed.call(Symbol()), `Built-in result for Symbol`);
