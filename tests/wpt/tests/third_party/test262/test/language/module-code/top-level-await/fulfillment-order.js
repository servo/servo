// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-async-module-execution-fulfilled
description: >
  When an async module fulfills, the promises relative to itself and its ancestors are resolved in leaf-to-root order
info: |
    AsyncModuleExecutionFulfilled ( module )
      ...
      7. If module.[[TopLevelCapability]] is not empty, then
        a. Assert: module.[[CycleRoot]] and module are the same Module Record.
        b. Perform ! Call(module.[[TopLevelCapability]].[[Resolve]], undefined, « undefined »).
      8. Let execList be a new empty List.
      9. Perform GatherAvailableAncestors(module, execList).
      10. Assert: All elements of execList have their [[AsyncEvaluationOrder]] field set to an integer, [[PendingAsyncDependencies]] field set to 0, and [[EvaluationError]] field set to empty.
      11. Let sortedExecList be a List whose elements are the elements of execList, sorted by their [[AsyncEvaluationOrder]] field in ascending order.
      12. For each Cyclic Module Record m of sortedExecList, do
        a. If m.[[Status]] is evaluated, then
          i. Assert: m.[[EvaluationError]] is not empty.
        b. Else if m.[[HasTLA]] is true, then
          i. Perform ExecuteAsyncModule(m).
        c. Else,
          i. Let result be m.ExecuteModule().
          ii. If result is an abrupt completion, then
            1. Perform AsyncModuleExecutionRejected(m, result.[[Value]]).
          iii. Else,
            1. Set m.[[AsyncEvaluationOrder]] to done.
            2. Set m.[[Status]] to evaluated.
            3. If m.[[TopLevelCapability]] is not empty, then
                a. Assert: m.[[CycleRoot]] and m are the same Module Record.
                b. Perform ! Call(m.[[TopLevelCapability]].[[Resolve]], undefined, « undefined »).
flags: [module, async]
features: [top-level-await, promise-with-resolvers]
includes: [compareArray.js]
---*/

import { p1, pA_start, pB_start } from "./fulfillment-order_setup_FIXTURE.js";

let logs = [];

const importsP = Promise.all([
  // Ensure that a.Evaluate() is called after b.Evaluate()
  pB_start.promise.then(() => import("./fulfillment-order_a_FIXTURE.js").finally(() => logs.push("A"))).catch(() => {}),
  import("./fulfillment-order_b_FIXTURE.js").finally(() => logs.push("B")).catch(() => {}),
]);

// Wait for evaluation of both graphs with entry points in A and B to start before
// settling the promise that B is blocked on.
Promise.all([pA_start.promise, pB_start.promise]).then(p1.resolve);

importsP.then(() => {
  assert.compareArray(logs, ["B", "A"]);

  $DONE();
});
