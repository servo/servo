// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Conversion of BigInt values to booleans
esid: sec-logical-not-operator-runtime-semantics-evaluation
info: |
  UnaryExpression: ! UnaryExpression

  1. Let expr be the result of evaluating UnaryExpression.
  2. Let oldValue be ToBoolean(? GetValue(expr)).
  3. If oldValue is true, return false.
  4. Return true.

  ToBoolean ( argument )

  BigInt: Return false if argument is 0n; otherwise return true.
features: [BigInt]
---*/

assert.sameValue(!0n, true, "!0n");
assert.sameValue(!1n, false, "!1n");
assert.sameValue(!-1n, false, "!-1n");
