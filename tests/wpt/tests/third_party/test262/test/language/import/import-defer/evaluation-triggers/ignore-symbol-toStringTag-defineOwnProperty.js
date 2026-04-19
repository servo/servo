// This file was procedurally generated from the following sources:
// - src/import-defer/defineOwnProperty.case
// - src/import-defer/trigger-on-possible-export/symbol-toStringTag.template
/*---
description: _ [[DefineOwnProperty]] (of Symbol.toStringTag, does not trigger execution)
esid: sec-module-namespace-exotic-objects
features: [import-defer]
flags: [generated, module]
info: |
    IsSymbolLikeNamespaceKey ( _P_, _O_ )
      1. If _P_ is a Symbol, return *true*.
      1. If _ns_.[[Deferred]] is *true* and _P_ is "then", return *true*.
      1. Return *false*.

    GetModuleExportsList ( _O_ )
      1. If _O_.[[Deferred]] is *true*, then
        1. Let _m_ be _O_.[[Module]].
        1. If _m_ is a Cyclic Module Record, _m_.[[Status]] is not ~evaluated~, and ReadyForSyncExecution(_m_) is *false*, throw a *TypeError* exception.
        1. Perform ? EvaluateSync(_m_).
      1. Return _O_.[[Exports]].


    [[DefineOwnProperty]] ( _P_, _Desc_ )
      1. If IsSymbolLikeNamespaceKey(_P_, _O_), return ! OrdinaryDefineOwnProperty(_O_, _Desc_).
      1. Let _current_ be ? _O_.[[GetOwnProperty]](_P_).
      1. NOTE: If _O_.[[Deferred]] is *true*, the step above will ensure that the module is evaluated.
      1. ...

---*/


import "./setup_FIXTURE.js";

import defer * as ns from "./dep_FIXTURE.js";

assert.sameValue(globalThis.evaluations.length, 0, "import defer does not trigger evaluation");

var key = Symbol.toStringTag;

try {
  Object.defineProperty(ns, key, { value: "hi" });
} catch (_) {}

assert.sameValue(globalThis.evaluations.length, 0, "It does not trigger evaluation");
