// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typeof-operator-runtime-semantics-evaluation
description: typeof Object (implements [[Call]]) === "function"
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    Return a String according to Table 35.

  #table-35

  Null "object"

---*/

assert.sameValue(
  typeof null,
   "object",
  'typeof null === "object"'
);

assert.sameValue(
  typeof RegExp("0").exec("1"),
   "object",
  'typeof RegExp("0").exec("1") === "object"'
);
