// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Parent completion orderings match the synchronous module behavior
info: |
  6.2.4 AsyncModuleExecutionFulfilled ( module )

  [...]
  5. Let _execList_ be a new empty List.
  6. Perform ! GatherAsyncParentCompletions(_module_, _execList_).
  7. Let _sortedExecList_ be a List of elements that are the elements of
     _execList_, in the order in which they had their [[AsyncEvaluating]]
     fields set to *true* in InnerModuleEvaluation.
  8. Assert: All elements of _sortedExecList_ have their [[AsyncEvaluating]]
     field set to *true*, [[PendingAsyncDependencies]] field set to 0 and
     [[EvaluationError]] field set to *undefined*.
  [...]

  Dependency graph for this test:

                             dfs-invariant.js
  .-----------------------------------+-------.
  |                                   |       v
  |                                   |       dfs-invariant-indirect_FIXTURE.js
  |                               .---|----------------------'
  v                               v   v
  dfs-invariant-direct-1_FIXTURE.js   dfs-invariant-direct-2_FIXTURE.js
            '--------.                            .--------'
                     v                            v
                     dfs-invariant-async_FIXTURE.js
esid: sec-moduleevaluation
flags: [module]
features: [top-level-await, globalThis]
---*/

import './dfs-invariant-direct-1_FIXTURE.js';
import './dfs-invariant-direct-2_FIXTURE.js';
import './dfs-invariant-indirect_FIXTURE.js';

assert.sameValue(globalThis.test262, 'async:direct-1:direct-2:indirect');
