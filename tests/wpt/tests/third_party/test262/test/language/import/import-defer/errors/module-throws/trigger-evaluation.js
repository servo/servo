// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-module-namespace-exotic-objects-get-p-receiver-EnsureDeferredNamespaceEvaluation
description: >
  Module evaluation errors are thrown
info: |
  [[Get]] ( _P_, _Receiver_ )
    1. ...
    1. If _O_.[[Deferred]] is **true**, perform ? EnsureDeferredNamespaceEvaluation(_O_).
    1. ...

  EnsureDeferredNamespaceEvaluation( _O_ )
    1. ...
    1. Perform ? EvaluateSync(_m_).
    1. ...

  EvaluateSync ( _module_ )
    1. ...
    1. Let _promise_ be ! _module_.Evaluate().
    1. Assert: _promise_.[[PromiseState]] is either ~fulfilled~ or ~rejected~.
    1. If _promise_.[[PromiseState]] is ~rejected~, then
      1. Return ThrowCompletion(_promise_.[[PromiseResult]]).
    1. ...

flags: [module]
features: [import-defer]
includes: [deepEqual.js]
---*/

import defer * as ns from "./throws_FIXTURE.js";

let err1, err2;
try { ns.foo } catch (e) { err1 = e };
assert.deepEqual(err1, { someError: "the error from throws_FIXTURE" }, "Evaluation errors are thrown when evaluating");

try { ns.foo } catch (e) { err2 = e };
assert.sameValue(err1, err2, "Evaluation errors are thrown for already evaluated modules");
