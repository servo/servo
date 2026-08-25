// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.delete
description: >
  Delete an entry from initial iterable.
info: |
  WeakSet.prototype.delete ( value )

  ...
  5. Let entries be the List that is the value of S’s [[WeakSetData]] internal
  slot.
  6. Repeat for each e that is an element of entries,
    a. If e is not empty and SameValue(e, value) is true, then
    i. Replace the element of entries whose value is e with an element whose
    value is empty.
    ii. Return true.
  ...
---*/

var foo = {};
var s = new WeakSet([foo]);

var result = s.delete(foo);

assert.sameValue(s.has(foo), false);
assert.sameValue(result, true, 'WeakSet#delete returns true');
