// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typeof-operator-runtime-semantics-evaluation
description: typeof Boolean literal
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    Return a String according to Table 35.

  #table-35

  Boolean "boolean"


---*/

assert.sameValue(
  typeof true,
   "boolean",
  'typeof true === "boolean"'
);

assert.sameValue(
  typeof false,
   "boolean",
  'typeof false === "boolean"'
);
