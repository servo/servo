// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-async-module-execution-rejected
description: >
  When an async module rejects, the promises relative to itself and its ancestors are resolved in leaf-to-root order
info: |
    AsyncModuleExecutionRejected ( module, error )
      ...
      9. If module.[[TopLevelCapability]] is not empty, then
        a. Assert: module.[[CycleRoot]] and module are the same Module Record.
        b. Perform ! Call(module.[[TopLevelCapability]].[[Reject]], undefined, « error »).
      10. For each Cyclic Module Record m of module.[[AsyncParentModules]], do
        a. Perform AsyncModuleExecutionRejected(m, error).
flags: [module, async]
features: [top-level-await, promise-with-resolvers]
includes: [compareArray.js]
---*/

import { p1, pA_start, pB_start } from "./rejection-order_setup_FIXTURE.js";

let logs = [];

const importsP = Promise.all([
  // Ensure that a.Evaluate() is called after b.Evaluate()
  pB_start.promise.then(() => import("./rejection-order_a_FIXTURE.js").finally(() => logs.push("A"))).catch(() => {}),
  import("./rejection-order_b_FIXTURE.js").finally(() => logs.push("B")).catch(() => {}),
]);

// Wait for evaluation of both graphs with entry points in A and B to start before
// rejecting the promise that B is blocked on.
Promise.all([pA_start.promise, pB_start.promise]).then(p1.reject);

importsP.then(() => {
  assert.compareArray(logs, ["B", "A"]);

  $DONE();
});
