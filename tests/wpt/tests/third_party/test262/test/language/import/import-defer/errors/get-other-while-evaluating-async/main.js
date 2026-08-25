// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-module-namespace-exotic-objects-get-p-receiver-EnsureDeferredNamespaceEvaluation
description: >
  Modules cannot try to trigger evaluation of modules that are in their ~evaluating-async~ phase
info: |
  10.4.6.8 [[Get]] ( _P_, _Receiver_ )
    1. ...
    1. If _O_.[[Deferred]] is **true**, perform ? EnsureDeferredNamespaceEvaluation(_O_).
    1. ...

  EnsureDeferredNamespaceEvaluation ( _O_ )
    1. Assert: _O_.[[Deferred]] is *false*.
    1. Let _m_ be _O_.[[Module]].
    1. If _m_ is a Cyclic Module Record, _m_.[[Status]] is not ~evaluated~, and ReadyForSyncExecution(_m_) is *false*, throw a *TypeError* exception.
    1. ...

  ReadyForSyncExecution( _module_, _seen_ )
    1. If _seen_ is not provided, let _seen_ be a new empty List.
    1. If _seen_ contains _module_, return *true*.
    1. Append _module_ to _seen_.
    1. If _module_.[[Status]] is ~evaluated~, return *true*.
    1. If _module_.[[Status]] is ~evaluating~ or ~evaluating-async~, return *false*.
    1. Assert: _module_.[[Status]] is ~linked~.
    1. If _module_.[[HasTLA]] is *true*, return *false*.
    1. For each ModuleRequest Record _required_ of _module_.[[RequestedModules]], do
      1. Let _requiredModule_ be GetImportedModule(_module_, _required_.[[Specifier]]).
      1. If ReadyForSyncExecution(_requiredModule_, _seen_) is *false*, then
        1. Return *false*.
    1. Return *true*.

flags: [module, async]
features: [import-defer, top-level-await]
includes: [asyncHelpers.js]
---*/

import { done } from "./promises_FIXTURE.js";
import "./dep-1-tla_FIXTURE.js";

asyncTest(async () => {
  await done;
  assert(globalThis["error on ns.foo while evaluating"] instanceof TypeError, "ns.foo while evaluating throws a TypeError");
  assert(globalThis["error on ns.foo while evaluating-async"] instanceof TypeError, "ns.foo while evaluating-async throws a TypeError");
  assert.sameValue(globalThis["value of ns.foo when evaluated"], 1);
});
