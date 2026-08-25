// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ContinueDynamicImport
description: >
  Dynamic import of an already errored module should fail
info: |
  ContinueDynamicImport ( _promiseCapability_, _moduleCompletion_ )
    1. ...
    1. Let _module_ be _moduleCompletion_.[[Value]].
    1. Let _loadPromise_ be _module_.LoadRequestedModules().
    1. Let _rejectedClosure_ be a new Abstract Closure with parameters (_reason_) that captures _promiseCapability_ and performs the following steps when called:
      1. Perform ! Call(_promiseCapability_.[[Reject]], *undefined*, « _reason_ »).
      1. Return ~unused~.
    1. Let _onRejected_ be CreateBuiltinFunction(_rejectedClosure_, 1, *""*, « »).
    1. Let _linkAndEvaluateClosure_ be a new Abstract Closure with no parameters that captures _module_, _promiseCapability_, and _onRejected_ and performs the following steps when called:
      1. Let _link_ be Completion(_module_.Link()).
      1. ...
      1. Let _evaluatePromise_ be _module_.Evaluate().
      1. Let _fulfilledClosure_ be a new Abstract Closure with no parameters that captures _module_ and _promiseCapability_ and performs the following steps when called:
        1. Let _namespace_ be GetModuleNamespace(_module_).
        1. Perform ! <emu-meta effects="user-code">Call</emu-meta>(_promiseCapability_.[[Resolve]], *undefined*, « _namespace_ »).
        1. Return ~unused~.
      1. Let _onFulfilled_ be CreateBuiltinFunction(_fulfilledClosure_, 0, *""*, « »).
      1. Perform PerformPromiseThen(_evaluatePromise_, _onFulfilled_, _onRejected_).
      1. Return ~unused~.
    1. Let _linkAndEvaluate_ be CreateBuiltinFunction(_linkAndEvaluateClosure_, 0, *""*, « »).
    1. Perform PerformPromiseThen(_loadPromise_, _linkAndEvaluate_, _onRejected_).
    1. Return ~unused~.

  _module_ . Evaluate (  )
    4. If _module_.[[TopLevelCapability]] is not ~empty~, then
       a. Return _module_.[[TopLevelCapability]].[[Promise]].

flags: [async]
features: [dynamic-import]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  await assert.throwsAsync(
    Error,
    () => import('./import-errored-module_FIXTURE.js'),
    'The import should reject (first import)'
  );
  await assert.throwsAsync(
    Error,
    () => import('./import-errored-module_FIXTURE.js'),
    'The import should reject (second import)'
  );
});
