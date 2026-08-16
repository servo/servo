// Copyright (C) 2026 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-IsModuleSCCEvaluated
description: >
  Deferred evaluation waits for an in-flight async cycle in the deferred graph
info: |
  In this test, the module A contains top-level await and forms a cycle with
  the module B, and the module Middle defers the module D, whose only
  dependency is B. This test first dynamically imports A and waits until A's
  evaluation has started and is suspended on the "blocker" promise, so A is
  the [[CycleRoot]] of the strongly connected component {A, B} and is
  guaranteed to still be EVALUATING-ASYNC. It then dynamically imports C,
  which imports Middle followed by ResolveBlocker. When Middle's evaluation
  starts, B is already EVALUATED, but its cycle root A is still
  EVALUATING-ASYNC, so Middle must wait for the whole cycle {A, B} instead of
  considering only B's individual status. ResolveBlocker then resolves the
  blocker, allowing A to finish; only after that may Middle's body execute,
  and accessing the deferred namespace of D must then only evaluate D itself.

  IsModuleSCCEvaluated ( _module_ )
    1. If _module_.[[CycleRoot]] is not EMPTY, then
      1. If _module_.[[CycleRoot]].[[Status]] is EVALUATED, return true.
      1. Return false.
    1. If _module_.[[Status]] is EVALUATED, return true.
    1. Return false.

  GatherAsynchronousTransitiveDependencies ( _module_, [ _seen_ ] )
    1. If _seen_ is not specified, let _seen_ be a new empty List.
    1. Let _result_ be a new empty List.
    1. If _seen_ contains _module_, return _result_.
    1. Append _module_ to _seen_.
    1. If _module_ is not a Cyclic Module Record, return _result_.
    1. If _module_.[[Status]] is either EVALUATING or IsModuleSCCEvaluated(_module_), return _result_.
    1. If _module_.[[HasTLA]] is *true*, then
      1. Append _module_ to _result_.
      1. Return _result_.
    1. For each ModuleRequest Record _required_ of _module_.[[RequestedModules]], do
      1. Let _requiredModule_ be GetImportedModule(_module_, _required_.[[Specifier]]).
      1. Let _additionalModules_ be GatherAsynchronousTransitiveDependencies(_requiredModule_, _seen_).
      1. For each Module Record _m_ of _additionalModules_, do
        1. If _result_ does not contain _m_, append _m_ to _result_.
    1. Return _result_.

  ReadyForSyncExecution ( _module_ [ , _seen_ ] )
    1. If _module_ is not a Cyclic Module Record, return true.
    1. If _seen_ is not present, set _seen_ to a new empty List.
    1. If _seen_ contains module, return true.
    1. Append _module_ to _seen_.
    1. If IsModuleSCCEvaluated(_module_), return true.
    1. If _module_.[[Status]] is EVALUATING or EVALUATING-ASYNC, return false.
    1. Assert: _module_.[[Status]] is LINKED.
    1. If _module_.[[HasTLA]] is true, return false.
    1. For each ModuleRequest Record request of _module_.[[RequestedModules]], do
       1. Let _requiredModule_ be GetImportedModule(_module_, _request_).
       1. If ReadyForSyncExecution(_requiredModule_, _seen_) is false, then
          1. Return false.
    1. Return true.
flags: [module, async]
features: [import-defer, top-level-await, dynamic-import, promise-with-resolvers]
---*/

import { aStarted } from "./setup_FIXTURE.js";

const pA = import("./a-tla_FIXTURE.js");

// Wait until A's evaluation has started and is suspended on the blocker, so
// that A is guaranteed to be EVALUATING-ASYNC when Middle starts evaluating.
await aStarted.promise;

const pC = import("./c_FIXTURE.js");

await Promise.all([pA, pC]);

assert.compareArray(globalThis.evaluations, [
  "B",
  "A-before-await",
  "resolve-blocker",
  "A-after-await",
  "Middle-before-nsD.z",
  "D",
  "Middle-after-nsD.z",
  "C",
]);

$DONE();
