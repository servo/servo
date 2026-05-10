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
  typeof Math.exp,
  "function",
  'typeof Math.exp === "function"'
);

assert.sameValue(
  typeof parseInt,
  "function",
  'typeof parseInt === "function"'
);

// TODO: should this be expanded to check all built-ins?
//        that might be excessive...
