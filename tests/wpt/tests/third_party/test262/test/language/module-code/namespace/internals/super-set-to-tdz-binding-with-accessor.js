// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ordinarysetwithowndescriptor
description: >
    super.foo = V calls the accessor on super class instead of throwing
    ReferenceError for TDZ binding
info: |
    OrdinarySetWithOwnDescriptor ( _O_, _P_, _V_, _Receiver_, _ownDesc_ )
      ...
      1. Assert: IsAccessorDescriptor(_ownDesc_) is *true*.
      1. Let _setter_ be _ownDesc_.[[Set]].
      1. If _setter_ is *undefined*, return *false*.
      1. Perform ? Call(_setter_, _Receiver_, « _V_ »).
      1. Return *true*.

    NOTE: When the super class has an accessor property, the setter is called
    directly and the Receiver's [[GetOwnProperty]] is never invoked, so the
    module namespace TDZ binding is not accessed.
flags: [module]
features: [let]
---*/

import * as ns from './super-set-to-tdz-binding-with-accessor.js';

var setterValue;

class A {
  constructor() { return ns; }
  set foo(v) { setterValue = v; }
};
class B extends A {
  constructor() {
    super();
    super.foo = 14;
  }
};

new B();

assert.sameValue(setterValue, 14, "setter on A.prototype should be called with the assigned value");

export let foo = 42;
