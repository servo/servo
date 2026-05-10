// Copyright (C) 2025 Sony Interactive Entertainment Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-errors-for-function-call-assignment-targets
description: Function call assignment target is still a runtime ReferenceError when the function is named `async`
info: |
  CallExpression :
    CoverCallExpressionAndAsyncArrowHead
    CallExpression Arguments
  1. If the host is a web browser or otherwise supports Runtime Errors for Function Call Assignment Targets, then
     a. If IsStrict(this CallExpression) is false, return ~web-compat~.
  2. Return ~invalid~.
flags: [noStrict]
---*/

function async() {}

assert.throws(ReferenceError, function() {
  async() = 1;
});

assert.throws(ReferenceError, function() {
  async() += 1;
});

assert.throws(ReferenceError, function() {
  async()++;
});

assert.throws(ReferenceError, function() {
  ++async();
});

assert.throws(ReferenceError, function() {
  for (async() in [1]) {}
});

assert.throws(ReferenceError, function() {
  for (async() of [1]) {}
});
