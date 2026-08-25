// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typeof-operator-runtime-semantics-evaluation
description: typeof String literal
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    Return a String according to Table 35.

  #table-35

  String "string"

---*/

assert.sameValue(
  typeof "1",
  "string",
  'typeof "1" === "string"'
);

assert.sameValue(
  typeof "NaN",
  "string",
  'typeof "NaN" === "string"'
);

assert.sameValue(
  typeof "Infinity",
  "string",
  'typeof "Infinity" === "string"'
);

assert.sameValue(
  typeof "",
  "string",
  'typeof "" === "string"'
);

assert.sameValue(
  typeof "true",
  "string",
  'typeof "true" === "string"'
);

assert.sameValue(
  typeof Date(),
  "string",
  'typeof Date() === "string"'
);
