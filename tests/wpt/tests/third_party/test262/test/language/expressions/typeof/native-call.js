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

  Object (implements [[Call]]) "function"


---*/

assert.sameValue(
  typeof new Function(),
   "function",
  'typeof new Function() === "function"'
);

assert.sameValue(
  typeof Function(),
   "function",
  'typeof Function() === "function"'
);

assert.sameValue(
  typeof Object,
   "function",
  'typeof Object === "function"'
);

assert.sameValue(
  typeof String,
   "function",
  'typeof String === "function"'
);

assert.sameValue(
  typeof Boolean,
   "function",
  'typeof Boolean === "function"'
);

assert.sameValue(
  typeof Number,
   "function",
  'typeof Number === "function"'
);

assert.sameValue(
  typeof Date,
   "function",
  'typeof Date === "function"'
);

assert.sameValue(
  typeof Error,
   "function",
  'typeof Error === "function"'
);

assert.sameValue(
  typeof RegExp,
   "function",
  'typeof RegExp === "function"'
);

// TODO: Should this be extended to include all built-ins?
