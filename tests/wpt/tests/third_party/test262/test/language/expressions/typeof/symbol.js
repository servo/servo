// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typeof-operator-runtime-semantics-evaluation
description: typeof Symbol() and Object(Symbol)
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    Return a String according to Table 35.

  #table-35

  Symbol "symbol"
  Object(Symbol()) "object"

features: [Symbol]
---*/

assert.sameValue(
  typeof Symbol(),
  "symbol",
  "typeof Symbol() === 'symbol'"
);

assert.sameValue(
  typeof Symbol("A"),
  "symbol",
  "typeof Symbol('A') === 'symbol'"
);

assert.sameValue(
  typeof Object(Symbol()),
  "object",
  "typeof Object(Symbol()) === 'object'"
);

assert.sameValue(
  typeof Object(Symbol("A")),
  "object",
  "typeof Object(Symbol('A')) === 'object'"
);

