// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Expression is a candidate for tail-call optimization.
esid: sec-static-semantics-hascallintailposition
info: |
  Expression Rules

  CoalesceExpression : CoalesceExpressionHead ?? BitwiseORExpression

  1. Return HasCallInTailPosition of BitwiseORExpression with argument call.
flags: [onlyStrict]
features: [tail-call-optimization, coalesce-expression]
includes: [tcoHelper.js]
---*/

var callCount = 0;
(function f(n) {
  if (n === 0) {
    callCount += 1
    return;
  }
  return null ?? f(n - 1);
}($MAX_ITERATIONS));
assert.sameValue(callCount, 1);
