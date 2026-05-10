// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ContinueDynamicImport
description: >
  `import.defer` causes eager evaluation of synchronous dependencies of async dependencies
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
        1. Let _asyncDepsEvaluationPromises_ be a new empty List.
        1. For each Module Record _dep_ of _evaluationList_, append _dep_.Evaluate() to _asyncDepsEvaluationPromises_.
        1. Let _iterator_ be CreateListIteratorRecord(_asyncDepsEvaluationPromises_).
        1. Let _pc_ be ! NewPromiseCapability(%Promise%).
        1. Let _evaluatePromise_ be ! PerformPromiseAll(_iterator_, %Promise%, _pc_, %Promise.resolve%).
      1. Else,
        1. ...
      1. Let _onFulfilled_ be CreateBuiltinFunction(_fulfilledClosure_, *""*, 0, &laquo; &raquo;).
      1. Perform PerformPromiseThen(_evaluatePromise_, _onFulfilled_, _onRejected_).
      1. Return ~UNUSED~.
    1. Let _linkAndEvaluate_ be CreateBuiltinFunction(_linkAndEvaluateClosure_, *""*, 0, &laquo; &raquo;).
    1. Perform PerformPromiseThen(_loadPromise_, _linkAndEvaluate_, _onRejected_).
    1. Return ~UNUSED~.

  GatherAsynchronousTransitiveDependencies ( _module_, [ _seen_ ] )
    1. If _seen_ is not specified, let _seen_ be a new empty List.
    1. Let _result_ be a new empty List.
    1. If _seen_ contains _module_, return _result_.
    1. Append _module_ to _seen_.
    1. If _module_ is not a Cyclic Module Record, return _result_.
    1. If _module_.[[Status]] is either ~evaluating~ or ~evaluated~, return _result_.
    1. If _module_.[[HasTLA]] is *true*, then
      1. Append _module_ to _result_.
      1. Return _result_.
    1. For each ModuleRequest Record _required_ of _module_.[[RequestedModules]], do
      1. Let _requiredModule_ be GetImportedModule(_module_, _required_.[[Specifier]]).
      1. Let _additionalModules_ be GatherAsynchronousTransitiveDependencies(_requiredModule_, _seen_).
      1. For each Module Record _m_ of _additionalModules_, do
        1. If _result_ does not contain _m_, append _m_ to _result_.
    1. Return _result_.

flags: [module, async]
features: [import-defer, top-level-await]
includes: [compareArray.js]
---*/

import "./setup_FIXTURE.js";

import.defer("./imports-tla-with-dep_FIXTURE.js").then(ns => {
  assert.compareArray(
    globalThis.evaluations,
    ["dep", "tla-with-dep start", "tla-with-dep end"]
  );
  ns.x;
  assert.compareArray(
    globalThis.evaluations,
    ["dep", "tla-with-dep start", "tla-with-dep end", "imports-tla-with-dep"]
  );
}).then($DONE, $DONE);
