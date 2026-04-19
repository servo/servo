// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref.prototype.deref
description: Return an Object target if weakRef.[[Target]] is not empty
info: |
  WeakRef.prototype.deref ()
  3. Return WeakRefDeref(_weakRef_).

  WeakRefDeref( _weakRef_ ):
  1. Let _target_ be _weakRef_.[[WeakRefTarget]].
  2. If _target_ is not ~empty~,
    a. Perform AddToKeptObjects(_target_).
    b. Return _target_.
features: [WeakRef]
---*/

var target = {};
var wref = new WeakRef(target);

assert.sameValue(wref.deref(), target, 'returns target');
assert.sameValue(wref.deref(), target, '[[Target]] is not emptied #1');
assert.sameValue(wref.deref(), target, '[[Target]] is not emptied #2');
assert.sameValue(wref.deref(), target, '[[Target]] is not emptied #3');
