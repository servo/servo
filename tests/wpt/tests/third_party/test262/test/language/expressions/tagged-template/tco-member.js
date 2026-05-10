// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Expression is a candidate for tail-call optimization.
esid: sec-static-semantics-hascallintailposition
flags: [onlyStrict]
features: [tail-call-optimization]
includes: [tcoHelper.js]
---*/

(function() {
  var finished = false;
  function f(_, n) {
    if (n === 0) {
      finished = true;
      return;
    }
    return f`${n-1}`;
  }
  f(null, $MAX_ITERATIONS);
  return finished;
}());
