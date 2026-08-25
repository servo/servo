// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function-calls-runtime-semantics-evaluation
description: >
  Tail-call with identifier named "eval" in function environment, local "eval" binding dynamically added.
info: |
  12.3.4.1 Runtime Semantics: Evaluation
    ...
    6. If Type(ref) is Reference and IsPropertyReference(ref) is false and
       GetReferencedName(ref) is "eval", then
      a. If SameValue(func, %eval%) is true, then
        ...
    ...
    9. Return ? EvaluateCall(func, ref, arguments, tailCall).

  12.3.4.2 Runtime Semantics: EvaluateCall( func, ref, arguments, tailPosition )
    ...
    7. If tailPosition is true, perform PrepareForTailCall().
    8. Let result be Call(func, thisValue, argList).
    ...

flags: [noStrict]
features: [tail-call-optimization]
includes: [tcoHelper.js]
---*/

var callCount = 0;

(function() {
  function f(n) {
    "use strict";
    if (n === 0) {
      callCount += 1
      return;
    }
    return eval(n - 1);
  }
  eval("var eval = f;");
  f($MAX_ITERATIONS);
})();

assert.sameValue(callCount, 1);
