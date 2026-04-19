// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typeof-operator-runtime-semantics-evaluation
description: Operator "typeof" uses GetValue
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    Set val to ? GetValue(val).
    ...

---*/

var count = 0;

Object.defineProperties(this, {
  x: {
    value: 1
  },
  y: {
    get() {
      count++;
      return 1;
    }
  }
});

assert.sameValue(
  typeof x,
   "number",
  'typeof x === "number"'
);

assert.sameValue(
  typeof y,
   "number",
  'typeof y === "number"'
);

assert.sameValue(count, 1);
