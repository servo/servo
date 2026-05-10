// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.delete
description: >
  Delete an entry that is an Object
info: |
  WeakSet.prototype.delete ( _value_ )
  4. Let _entries_ be the List that is _S_.[[WeakSetData]].
  5. For each element _e_ of _entries_, do
    a. If _e_ is not ~empty~ and SameValue(_e_, _value_) is *true*, then
      i. Replace the element of _entries_ whose value is _e_ with an element
        whose value is ~empty~.
      ii. Return *true*.
features: [WeakSet]
---*/

var foo = {};
var s = new WeakSet();

s.add(foo);

var result = s.delete(foo);

assert.sameValue(s.has(foo), false);
assert.sameValue(result, true, 'WeakSet#delete returns true');
