// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ContinueDynamicImport
description: >
  `import.defer` of a sync module graph does not cause evaluation
info: |
  13.3.10.1.1 ContinueDynamicImport (
    _payload_: a DynamicImportState Record,
    _moduleCompletion_: either a normal completion containing a Module Record or a throw completion,
  ): ~UNUSED~
    1. Let _promiseCapability_ be _payload_.[[PromiseCapability]].
    1. Let _phase_ be _payload_.[[Phase]].
    1. ...
    1. Let _module_ be _moduleCompletion_.[[Value]].
    1. Let _loadPromise_ be _module_.LoadRequestedModules().
    1. ...
    1. Let _linkAndEvaluateClosure_ be a new Abstract Closure with no parameters that captures _module_, _promiseCapability_, _phase_ and _onRejected_ and performs the following steps when called:
      1. ...
      1. Let _fulfilledClosure_ be a new Abstract Closure with no parameters that captures _module_, _phase_, and _promiseCapability_ and performs the following steps when called:
        1. Let _namespace_ be GetModuleNamespace(_module_, _phase_).
        1. Perform ! Call(_promiseCapability_.[[Resolve]], *undefined*, &laquo; _namespace_ &raquo;).
        1. Return ~UNUSED~.
      1. If _phase_ is ~defer~, then
        1. Let _evaluationList_ be GatherAsynchronousTransitiveDependencies(_module_).
        1. If _evaluationList_ is empty, then
          1. Perform _fulfilledClosure_().
          1. Return ~UNUSED~.
      1. ...
    1. Let _linkAndEvaluate_ be CreateBuiltinFunction(_linkAndEvaluateClosure_, *""*, 0, &laquo; &raquo;).
    1. Perform PerformPromiseThen(_loadPromise_, _linkAndEvaluate_, _onRejected_).
    1. Return ~UNUSED~.

flags: [module, async]
features: [import-defer]
includes: [compareArray.js]
---*/

import "./setup_FIXTURE.js";

import.defer("./sync_FIXTURE.js").then(ns => {
  assert.compareArray(globalThis.evaluations, []);
  ns.x;
  assert.compareArray(globalThis.evaluations, ["dep", "sync"]);
}).then($DONE, $DONE);
