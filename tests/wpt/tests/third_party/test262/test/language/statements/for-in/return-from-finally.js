// Copyright (C) 2026 Luna Pfeiffer. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-forinofloopevaluation
description: >
    Control flow during body evaluation should honor `return` statements within
    the `finally` block of `try` statements.
---*/

var obj = { name: "Luna" };
var i = 0;

var result = (function() {
  for (var x in obj) {
    try {
    } finally {
      i++;
      return 42;

      $DONOTEVALUATE();
    }

    $DONOTEVALUATE();
  }

  $DONOTEVALUATE();
})();

assert.sameValue(result, 42);
assert.sameValue(i, 1);
