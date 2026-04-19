// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typeof-operator-runtime-semantics-evaluation
description: If IsUnresolvableReference(val) is true, return "undefined".
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    If Type(val) is Reference, then
      If IsUnresolvableReference(val) is true, return "undefined".
    ...

---*/

assert.sameValue(
  typeof x,
  "undefined",
  "typeof x === 'undefined'"
);
