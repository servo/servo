// Copyright (C) 2019 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-return-statement-runtime-semantics-evaluation
description: >
  Return with an explicit return value awaits this value.
info: |
  13.10.1 Runtime Semantics: Evaluation

    ReturnStatement : return;
      1. Return Completion { [[Type]]: return, [[Value]]: undefined, [[Target]]: empty }.

    ReturnStatement : return Expression ;
      1. Let exprRef be the result of evaluating Expression.
      2. Let exprValue be ? GetValue(exprRef).
      3. If ! GetGeneratorKind() is async, set exprValue to ? Await(exprValue).
      4. Return Completion { [[Type]]: return, [[Value]]: exprValue, [[Target]]: empty }.

  25.5.3.2 AsyncGeneratorStart ( generator, generatorBody )

    ...
    5. Set the code evaluation state of genContext such that when evaluation is resumed for that
       execution context the following steps will be performed:
      a. Let result be the result of evaluating generatorBody.
      ...
      e. If result is a normal completion, let resultValue be undefined.
      ...
      g. Return ! AsyncGeneratorResolve(generator, resultValue, true).

includes: [compareArray.js]
flags: [async]
features: [async-iteration]
---*/

// 25.5.3.2, step 5.e: |generatorBody| execution ends with a normal completion.
async function* g1() {
  // no return
}

// 13.10.1: No expression form means direct return.
async function* g2() {
  return;
}

// 13.10.1: Explicit expression requires Await.
async function* g3() {
  return undefined; // Return undefined via global value `undefined`.
}

// 13.10.1: Explicit expression requires Await.
async function* g4() {
  return void 0; // Return undefined via void expression.
}

var expected = [
  "tick 1",

  "g1 ret",
  "g2 ret",

  "tick 2",

  "g3 ret",
  "g4 ret",
];

var actual = [];

Promise.resolve(0)
  .then(() => actual.push("tick 1"))
  .then(() => actual.push("tick 2"))
  .then(() => {
    assert.compareArray(actual, expected, "Ticks for implicit and explicit return undefined");
}).then($DONE, $DONE);

g1().next().then(v => actual.push("g1 ret"));
g2().next().then(v => actual.push("g2 ret"));
g3().next().then(v => actual.push("g3 ret"));
g4().next().then(v => actual.push("g4 ret"));
