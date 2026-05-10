// Copyright (C) 2016 Rick Waldron, André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Rick Waldron, André Bargull
esid: sec-exp-operator-runtime-semantics-evaluation
description: Exponentiation Operator expression order of evaluation
info: |
    ExponentiationExpression:
      UpdateExpression ** ExponentiationExpression

    1. Let left be the result of evaluating UpdateExpression.
    2. Let leftValue be ? GetValue(left).
    3. Let right be the result of evaluating ExponentiationExpression.
    4. Let rightValue be ? GetValue(right).
    5. Let base be ? ToNumber(leftValue).
    6. Let exponent be ? ToNumber(rightValue).
    7. Return the result of Applying the ** operator with base and exponent as specified in 12.7.3.4.
features: [exponentiation]
---*/

var capture = [];
var leftValue = { valueOf() { capture.push("leftValue"); return 3; }};
var rightValue = { valueOf() { capture.push("rightValue"); return 2; }};

(capture.push("left"), leftValue) ** (capture.push("right"), rightValue);

// Expected per operator evaluation order: "left", "right", "leftValue", "rightValue"

assert.sameValue(capture[0], "left", "Expected the 1st element captured to be 'left'");
assert.sameValue(capture[1], "right", "Expected the 2nd element captured to be 'right'");
assert.sameValue(capture[2], "leftValue", "Expected the 3rd element captured to be 'leftValue'");
assert.sameValue(capture[3], "rightValue", "Expected the 4th element captured to be 'rightValue'");
