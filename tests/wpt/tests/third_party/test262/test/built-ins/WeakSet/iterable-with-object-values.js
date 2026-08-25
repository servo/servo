// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-iterable
description: >
  Returns the new WeakSet adding Object values from the iterable parameter.
info: |
  WeakSet ( [ _iterable_ ] )
  8. Repeat,
    d. Let _status_ be Completion(Call(_adder_, _set_, « _nextValue_ »)).

  WeakSet.prototype.add ( _value_ ):
  6. Append _value_ as the last element of _entries_.
features: [WeakSet]
---*/

var first = {};
var second = {};
var added = [];
var add = WeakSet.prototype.add;
WeakSet.prototype.add = function(value) {
  added.push(value);
  return add.call(this, value);
};
var s = new WeakSet([first, second]);

assert.sameValue(added.length, 2, 'Called WeakSet#add for each object');
assert.sameValue(added[0], first, 'Adds object in order - first');
assert.sameValue(added[1], second, 'Adds object in order - second');
