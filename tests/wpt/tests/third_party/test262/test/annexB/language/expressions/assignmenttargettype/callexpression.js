// Copyright (C) 2025 Sony Interactive Entertainment Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-errors-for-function-call-assignment-targets
description: Function call LHS in `=` assignment is a runtime ReferenceError
info: |
  CallExpression :
    CoverCallExpressionAndAsyncArrowHead
    CallExpression Arguments
  1. If the host is a web browser or otherwise supports Runtime Errors for Function Call Assignment Targets, then
     a. If IsStrict(this CallExpression) is false, return ~web-compat~.
  2. Return ~invalid~.
flags: [noStrict]
---*/

var fCalled = false;
var fValueOfCalled = false;
function f() {
  fCalled = true;
  return {
    valueOf() { fValueOfCalled = true; return 1; }
  };
}

var gCalled = false;
function g() {
  gCalled = true;
  return 1;
}

assert.throws(ReferenceError, function() {
  f() = g();
});

assert(fCalled);
assert(!fValueOfCalled);
assert(!gCalled);
