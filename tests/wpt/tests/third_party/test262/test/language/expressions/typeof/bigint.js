// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typeof-operator-runtime-semantics-evaluation
description: typeof BigInt literal and BigInt object
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    Return a String according to Table 35.

  #table-35

  BigInt "bigint"
  Object(BigInt()) "object"

features: [BigInt]
---*/

assert.sameValue(
  typeof 0n,
  "bigint",
  "typeof 0n === 'bigint'"
);
assert.sameValue(
  typeof BigInt(0n),
  "bigint",
  "typeof BigInt(0n) === 'bigint'"
);
assert.sameValue(
  typeof BigInt(0),
  "bigint",
  "typeof BigInt(0) === 'bigint'"
);
assert.sameValue(
  typeof Object(BigInt(0n)),
  "object",
  "typeof Object(BigInt(0n)) === 'object'"
);
assert.sameValue(
  typeof Object(BigInt(0)),
  "object",
  "typeof Object(BigInt(0)) === 'object'"
);
assert.sameValue(
  typeof Object(0n),
  "object",
  "typeof Object(0n) === 'object'"
);
