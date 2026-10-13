// Copyright (C) 2026 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-moduleevaluation
description: >
  Dynamic import of a fulfilled member of an errored async cycle rejects with
  the recorded evaluation error
info: |
  When the top-level-await cycle {A, B, C} rejects (B throws after its await),
  only B and its async parent modules (X and main) record the evaluation
  error. A and C had already finished executing and stay EVALUATED with an
  empty [[EvaluationError]]. The cycle is first evaluated starting from B (the
  first dependency of main), so B is the [[CycleRoot]] of {A, B, C} and, not
  being an evaluation entry point itself, it has no [[TopLevelCapability]].

  A later dynamic import of C redirects to its [[CycleRoot]] B, whose
  [[Status]] is EVALUATED with a non-empty [[EvaluationError]] and no
  [[TopLevelCapability]]. Evaluate() is well-defined on such case, thus a
  fresh capability is created and installed as B's [[TopLevelCapability]],
  InnerModuleEvaluation returns the recorded [[EvaluationError]], and the
  capability is rejected with it. This makes the import of C reject with
  the same error with which the cycle originally failed.

  Evaluate ( )
    1. ...
    1. If _module_.[[Status]] is either EVALUATING-ASYNC or EVALUATED, then
      1. If module.[[CycleRoot]] is not empty, then
        1. Set _module_ to _module_.[[CycleRoot]].
      1. Else,
        1. ...
    1. If _module_.[[TopLevelCapability]] is not EMPTY, then
      1. Return _module_.[[TopLevelCapability]].[[Promise]].
    1. Let _stack_ be a new empty List.
    1. Let _capability_ be ! NewPromiseCapability(%Promise%).
    1. Set _module_.[[TopLevelCapability]] to _capability_.
    1. Let _result_ be Completion(InnerModuleEvaluation(_module_, _stack_, 0)).
    1. If _result_ is an abrupt completion, then
      1. ...
      1. Perform ! Call(_capability_.[[Reject]], *undefined*, « _result_.[[Value]] »).
    1. ...
    1. Return _capability_.[[Promise]].

  InnerModuleEvaluation ( _module_, _stack_, _index_ )
    1. ...
    1. If _module_.[[Status]] is either EVALUATING-ASYNC or EVALUATED, then
      1. If _module_.[[EvaluationError]] is EMPTY, return _index_.
      1. Return ? _module_.[[EvaluationError]].
    1. ...
flags: [async]
features: [top-level-await, dynamic-import]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  var errorFromMain = null;
  try {
    await import("./import-fulfilled-member-of-errored-cycle-main_FIXTURE.js");
  } catch (err) {
    errorFromMain = err;
  }
  assert.notSameValue(errorFromMain, null, "The import of main should reject");
  assert.sameValue(
    errorFromMain instanceof Error,
    true,
    "The import of main should reject with the error thrown in B"
  );
  assert.sameValue(errorFromMain.message, "async error in B");

  var errorFromC = null;
  try {
    await import("./import-fulfilled-member-of-errored-cycle-c_FIXTURE.js");
  } catch (err) {
    errorFromC = err;
  }
  assert.notSameValue(errorFromC, null, "The import of C should reject");
  assert.sameValue(
    errorFromC,
    errorFromMain,
    "The import of C should reject with the [[EvaluationError]] recorded for the cycle root B"
  );
});
