// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typeof-operator-runtime-semantics-evaluation
description: typeof Object (standard exotic and does not implement [[Call]]) === "object"
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    Return a String according to Table 35.

  #table-35

  Object (standard exotic and does not implement [[Call]]) "object"


---*/

assert.sameValue(
  typeof this,
   "object",
  'typeof this === "object"'
);

assert.sameValue(
  typeof new Object(),
   "object",
  'typeof new Object() === "object"'
);

assert.sameValue(
  typeof new Array(),
   "object",
  'typeof new Array() === "object"'
);

assert.sameValue(
  typeof new String(),
   "object",
  'typeof new String() === "object"'
);

assert.sameValue(
  typeof new Boolean(),
   "object",
  'typeof new Boolean() === "object"'
);

assert.sameValue(
  typeof new Number(),
   "object",
  'typeof new Number() === "object"'
);

assert.sameValue(
  typeof new Date(0),
   "object",
  'typeof new Date(0) === "object"'
);

assert.sameValue(
  typeof new Error(),
   "object",
  ' typeof new Error() === "object"'
);

assert.sameValue(
  typeof new RegExp(),
   "object",
  ' typeof new RegExp() === "object"'
);

