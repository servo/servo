// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.towellformed
description: >
  The method should coerce the receiver to a string.
info: |
  String.prototype.toWellFormed ( )

  2. Let S be ? ToString(O).
  …

features: [String.prototype.toWellFormed]
---*/

const tests = [
  [true, "true", Boolean.prototype],
  [1, "1", Number.prototype],
  [1n, "1", BigInt.prototype],
];

for (const [v, expected, proto] of tests) {
  proto.toString = function() {
    throw new Test262Error(`should not call toString on the prototype for ${typeof v}`);
  }
  let result = String.prototype.toWellFormed.call(v);
  delete proto.toString;
  assert.sameValue(result, expected, `toWellFormed for ${typeof v}`);
}

Symbol.prototype.toString = function() { throw new TypeError("should not call toString on the prototype for Symbol"); }
assert.throws(TypeError, () => String.prototype.toWellFormed.call(Symbol()), `Built-in result for Symbol`);
