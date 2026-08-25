// This file was procedurally generated from the following sources:
// - src/import-defer/super-property-set-exported.case
// - src/import-defer/trigger-on-possible-export/then-exported.template
/*---
description: _ [[GetOwnProperty]] called on super access (of 'then' when it is an exported name, does not trigger execution)
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


    SuperProperty : super [ Expression ]
      1. Let _env_ be GetThisEnvironment().
      1. Let _actualThis_ be ? _env_.GetThisBinding().
      1. Let _propertyNameReference_ be ? Evaluation of |Expression|.
      1. Let _propertyNameValue_ be ? GetValue(_propertyNameReference_).
      1. Let _strict_ be IsStrict(this |SuperProperty|).
      1. Return MakeSuperPropertyReference(_actualThis_, _propertyNameValue_, _strict_).

    MakeSuperPropertyReference ( _actualThis_, _propertyKey_, _strict_ )
      1. Let _env_ be GetThisEnvironment().
      1. Assert: _env_.HasSuperBinding() is *true*.
      1. Assert: _env_ is a Function Environment Record.
      1. Let _baseValue_ be GetSuperBase(_env_).
      1. Return the Reference Record { [[Base]]: _baseValue_, [[ReferencedName]]: _propertyKey_, [[Strict]]: _strict_, [[ThisValue]]: _actualThis_ }.

    PutValue ( _V_, _W_ )
      1. If _V_ is not a Reference Record, throw a *ReferenceError* exception.
      ...
      1. If IsPropertyReference(_V_) is *true*, then
        1. Let _baseObj_ be ? ToObject(_V_.[[Base]]).
        ...
        1. Let _succeeded_ be ? _baseObj_.[[Set]](_V_.[[ReferencedName]], _W_, GetThisValue(_V_)).
        1. If _succeeded_ is *false* and _V_.[[Strict]] is *true*, throw a *TypeError* exception.
        1. Return ~unused~.
      ...

    OrdinarySetWithOwnDescriptor ( _O_, _P_, _V_, _Receiver_, _ownDesc_ )
      1. If _ownDesc_ is *undefined*, then
        1. Let _parent_ be ? _O_.[[GetPrototypeOf]]().
        1. If _parent_ is not *null*, return ? _parent_.[[Set]](_P_, _V_, _Receiver_).
        1. Set _ownDesc_ to the PropertyDescriptor { [[Value]]: *undefined*, [[Writable]]: *true*, [[Enumerable]]: *true*, [[Configurable]]: *true* }.
      1. If IsDataDescriptor(_ownDesc_) is *true*, then
        1. If _ownDesc_.[[Writable]] is *false*, return *false*.
        1. If _Receiver_ is not an Object, return *false*.
        1. Let _existingDescriptor_ be ? _Receiver_.[[GetOwnProperty]](_P_).
        1. If _existingDescriptor_ is *undefined*, then
          1. Assert: _Receiver_ does not currently have a property _P_.
          1. Return ? CreateDataProperty(_Receiver_, _P_, _V_).
        1. If IsAccessorDescriptor(_existingDescriptor_) is *true*, return *false*.
        1. If _existingDescriptor_.[[Writable]] is *false*, return *false*.
        1. Let _valueDesc_ be the PropertyDescriptor { [[Value]]: _V_ }.
        1. Return ? _Receiver_.[[DefineOwnProperty]](_P_, _valueDesc_).
      ...

---*/


import "./setup_FIXTURE.js";

import defer * as ns from "./dep-then_FIXTURE.js";

assert.sameValue(globalThis.evaluations.length, 0, "import defer does not trigger evaluation");

var key = "then";

class A { constructor() { return ns; } };
class B extends A {
  constructor() {
    super();
    super[key] = 14;
  }
};

try {
  new B();
} catch (_) {}

assert.sameValue(globalThis.evaluations.length, 0, "It does not trigger evaluation");
