// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-getownproperty-p
description: >
    super.foo = V throws ReferenceError when Receiver is a module namespace
    with an uninitialized binding
info: |
    OrdinarySetWithOwnDescriptor ( _O_, _P_, _V_, _Receiver_, _ownDesc_ )
      ...
      1. If IsDataDescriptor(_ownDesc_) is *true*, then
        1. If _ownDesc_.[[Writable]] is *false*, return *false*.
        1. If _Receiver_ is not an Object, return *false*.
        1. Let _existingDescriptor_ be ? _Receiver_.[[GetOwnProperty]](_P_).
        ...

    [[GetOwnProperty]] ( _P_ )
      ...
      1. Let _value_ be ? _O_.[[Get]](_P_, _O_).
      ...

    [[Get]] ( _P_, _Receiver_ )
      ...
      1. Let _targetEnvRec_ be _targetEnv_'s EnvironmentRecord.
      1. Return ? _targetEnvRec_.GetBindingValue(_binding_.[[BindingName]], *true*).

    NOTE: If the binding is uninitialized, GetBindingValue throws a *ReferenceError*.
flags: [module]
features: [let]
---*/

import * as ns from './super-access-to-tdz-binding.js';

class A { constructor() { return ns; } };
class B extends A {
  constructor() {
    super();
    super.foo = 14;
  }
};

assert.throws(ReferenceError, function() {
  new B();
});

export let foo = 42;
