// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  An implementation may unobservably reset [[ModuleAsyncEvaluationCount]] to 0
  whenever there are no pending modules.
info: |
  IncrementModuleAsyncEvaluationCount ( )
    1. Let AR be the Agent Record of the surrounding agent.
    2. Let count be AR.[[ModuleAsyncEvaluationCount]].
    3. Set AR.[[ModuleAsyncEvaluationCount]] to count + 1.
    4. Return count.

    NOTE: This value is only used to keep track of the relative evaluation order
    between pending modules. An implementation may unobservably reset
    [[ModuleAsyncEvaluationCount]] to 0 whenever there are no pending modules.

  InnerModuleEvaluation ( module, stack, index )
    ...
    12. If module.[[PendingAsyncDependencies]] > 0 or module.[[HasTLA]] is true, then
      a. Assert: module.[[AsyncEvaluationOrder]] is unset.
      b. Set module.[[AsyncEvaluationOrder]] to IncrementModuleAsyncEvaluationCount().
      ...

  AsyncModuleExecutionFulfilled ( module )
    ...
     9. Perform GatherAvailableAncestors(module, execList).
    10. ...
    11. Let sortedExecList be a List whose elements are the elements of execList, sorted by their [[AsyncEvaluationOrder]] field in ascending order.
    12. For each Cyclic Module Record m of sortedExecList, do
      a. If m.[[Status]] is evaluated, then
        i. Assert: m.[[EvaluationError]] is not empty.
      b. Else if m.[[HasTLA]] is true, then
        i. Perform ExecuteAsyncModule(m).
      c. Else,
        i. Let result be m.ExecuteModule().
        ...

  Module graph (the order of dependencies in each module is important, and it's left-to-right):
        ┌─────┐    ┌─────┐    ┌─────┐
        │  A  │    │  C  │    │  D  │
        └─────┘    └─────┘    └─────┘
           │                   │   │
           │                   ▼   │
           │              ┌─────┐  │
           │              │  E  │  │
           │              └─────┘  │
           │    ┌──────────────────┘
           ▼    ▼
         ┌───────┐
         │   B   │
         └───────┘

  Where B and C have top-level await. The test orchestrates the evaluation order such that:
  - Import A first
  - Once B starts evaluating, import C and immediately resolve its top-level await
  - Once C finishes evaluating, import D
  - Once E is evaluated, resolve B's await

esid: sec-IncrementModuleAsyncEvaluationCount
flags: [module, async]
features: [top-level-await, dynamic-import, promise-with-resolvers]
includes: [compareArray.js]
---*/

import { logs, pB, pB_start, pE_start } from "./unobservable-global-async-evaluation-count-reset-setup_FIXTURE.js";

const pA = import("./unobservable-global-async-evaluation-count-reset-a_FIXTURE.js");
let pD;

pB_start.promise.then(() => {
  return import("./unobservable-global-async-evaluation-count-reset-c_FIXTURE.js");
}).then(() => {
  pD = import("./unobservable-global-async-evaluation-count-reset-d_FIXTURE.js");
  return pE_start.promise;
}).then(() => {
  pB.resolve();
  return Promise.all([pA, pD]);
}).then(() => {
  assert.compareArray(logs, ["A", "D"], "A should evaluate before D");
}).then($DONE, $DONE);
