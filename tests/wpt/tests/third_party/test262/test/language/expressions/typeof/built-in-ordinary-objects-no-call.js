// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typeof-operator-runtime-semantics-evaluation
description: typeof Object (ordinary and does not implement [[Call]]) === "object"
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    Return a String according to Table 35.

  #table-35

  Object (ordinary and does not implement [[Call]]) "object"

---*/

assert.sameValue(
  typeof Math,
   "object",
  'typeof Math === "object"'
);

assert.sameValue(
  typeof Reflect,
   "object",
  'typeof Reflect === "object"'
);
