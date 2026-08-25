// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-innermoduleevaluation
description: >
  A module depending on an async module of a separate cycle should wait for the cycle root to complete
info: |
  Module graph:

                ┌──────────────────┐
                │   entrypoint     │
                └──────────────────┘
                    │           │
                    ▼           ▼
    ┌──────────────────┐     ┌────────────────────────┐
    │ cycle root (TLA) │     │ importer of cycle leaf │
    └──────────────────┘     └────────────────────────┘
           │    ▲               │
           ▼    │               │
    ┌──────────────────┐        │
    │ cycle leaf (TLA) │ ◄──────┘
    └──────────────────┘

  This test exercises step 11.c.iv.1 of the following algorithm when _module_ is
  "importer of cycle leaf", _requiredModule_ is "cycle leaf (TLA)", and
  _requiredModule_.[[CycleRoot]] is "cycle root (TLA)".
  The [[Status]] of "cycle leaf (TLA)" and of "cycle root (TLA)" is ~evaluating-async~,
  because they have already been traversed and they are blocked on the TLA in "cycle leaf (TLA)".
  Thus, their [[AsyncEvaluationOrder]] is an integer, so the _requiredModule_ variable is used
  to determine what module "importer of cycle leaf" should wait for.

    InnerModuleEvaluation ( module, stack, index )
        ...
        11. For each ModuleRequest Record request of module.[[RequestedModules]], do
            a. Let requiredModule be GetImportedModule(module, request).
            b. Set index to ? InnerModuleEvaluation(requiredModule, stack, index).
            c. If requiredModule is a Cyclic Module Record, then
                i. Assert: requiredModule.[[Status]] is one of evaluating, evaluating-async, or evaluated.
                ii. Assert: requiredModule.[[Status]] is evaluating if and only if stack contains requiredModule.
                iii. If requiredModule.[[Status]] is evaluating, then
                    1. Set module.[[DFSAncestorIndex]] to min(module.[[DFSAncestorIndex]], requiredModule.[[DFSAncestorIndex]]).
                iv. Else,
                    1. Set requiredModule to requiredModule.[[CycleRoot]].
                    2. Assert: requiredModule.[[Status]] is either evaluating-async or evaluated.
                    3. If requiredModule.[[EvaluationError]] is not empty, return ? requiredModule.[[EvaluationError]].
                v. If requiredModule.[[AsyncEvaluationOrder]] is an integer, then
                    1. Set module.[[PendingAsyncDependencies]] to module.[[PendingAsyncDependencies]] + 1.
                    2. Append module to requiredModule.[[AsyncParentModules]].

flags: [module, async]
features: [top-level-await]
includes: [compareArray.js]
---*/

import "./pending-async-dep-from-cycle_setup_FIXTURE.js";
import "./pending-async-dep-from-cycle_cycle-root_FIXTURE.js";
import "./pending-async-dep-from-cycle_import-cycle-leaf_FIXTURE.js";

assert.compareArray(globalThis.logs, [
  "cycle leaf start",
  "cycle leaf end",
  "cycle root start",
  // Without the step covered by this test,
  // these last two entries would be swapped.
  "cycle root end",
  "importer of cycle leaf"
]);

$DONE();
