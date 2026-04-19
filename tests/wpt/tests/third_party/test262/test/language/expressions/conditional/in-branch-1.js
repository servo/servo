// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-conditional-operator
es6id: 12.13
description: >
  The first AssignmentExpression may include the `in` keyword in any context
info: |
  Syntax

  ConditionalExpression[In, Yield] :
    LogicalORExpression[?In, ?Yield]
    LogicalORExpression[?In, ?Yield] ? AssignmentExpression[+In, ?Yield] : AssignmentExpression[?In, ?Yield]
---*/

var cond1Count = 0;
var cond2Count = 0;
var cond1 = function() {
  cond1Count += 1;
  return {};
};
var cond2 = function() {
  cond2Count += 1;
};
for (true ? '' in cond1() : cond2(); false; ) ;

assert.sameValue(cond1Count, 1);
assert.sameValue(cond2Count, 0);
