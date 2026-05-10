// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref.prototype.deref
description: Return target if weakRef.[[Target]] is not empty (applying custom this)
info: |
  WeakRef.prototype.deref ()

  1. Let weakRef be the this value.
  ...
  4. Let target be the value of weakRef.[[Target]].
  5. If target is not empty,
    a. Perform ! KeepDuringJob(target).
    b. Return target.
  6. Return undefined.
features: [WeakRef]
---*/

var target = {};
var deref = WeakRef.prototype.deref;
var wref = new WeakRef(target);

assert.sameValue(deref.call(wref), target, 'returns target');
assert.sameValue(deref.call(wref), target, '[[Target]] is not emptied #1');
assert.sameValue(deref.call(wref), target, '[[Target]] is not emptied #2');
assert.sameValue(deref.call(wref), target, '[[Target]] is not emptied #3');
