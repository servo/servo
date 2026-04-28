// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref.prototype.deref
description: Return a Symbol target if weakRef.[[Target]] is not empty
info: |
  WeakRef.prototype.deref ()
  3. Return WeakRefDeref(_weakRef_).

  WeakRefDeref( _weakRef_ ):
  1. Let _target_ be _weakRef_.[[WeakRefTarget]].
  2. If _target_ is not ~empty~,
    a. Perform AddToKeptObjects(_target_).
    b. Return _target_.
features: [Symbol, WeakRef, symbols-as-weakmap-keys]
---*/

var target = Symbol('a description');
var wref = new WeakRef(target);

assert.sameValue(wref.deref(), target, 'returns a regular symbol target');

var wref2 = new WeakRef(Symbol.hasInstance);

assert.sameValue(wref2.deref(), Symbol.hasInstance, 'returns a well-known symbol target');
