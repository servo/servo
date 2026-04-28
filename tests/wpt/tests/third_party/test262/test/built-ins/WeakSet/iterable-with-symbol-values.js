// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-iterable
description: >
  Returns the new WeakSet adding Symbol values from the iterable parameter.
info: |
  WeakSet ( [ _iterable_ ] )
  8. Repeat,
    d. Let _status_ be Completion(Call(_adder_, _set_, « _nextValue_ »)).

  WeakSet.prototype.add ( _value_ ):
  6. Append _value_ as the last element of _entries_.
features: [Symbol, WeakSet, symbols-as-weakmap-keys]
includes: [compareArray.js]
---*/

var first = Symbol('a description');
var second = Symbol('a description');
var added = [];
var realAdd = WeakSet.prototype.add;
WeakSet.prototype.add = function(value) {
  added.push(value);
  return realAdd.call(this, value);
};
var s = new WeakSet([first, second, Symbol.hasInstance]);

assert.compareArray(
  added,
  [first, second, Symbol.hasInstance],
  "add() was called 3 times, on the two unregistered and one well-known symbols in order"
);

WeakSet.prototype.add = realAdd;
