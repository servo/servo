// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: >
  Returns the new WeakMap adding entries from the iterable parameter, with
  Symbol keys.
info: |
  WeakMap ( [ _iterable_ ] )
  5. Let _adder_ be ? Get(_map_, *"set"*).
  6. Return ? AddEntriesFromIterable(_map_, _iterable_, _adder_).

  AddEntriesFromIterable:
  3. Repeat,
    i. Let _status_ be Completion(Call(_adder_, _target_, « _k_, _v_ »)).

  WeakMap.prototype.set( _key_, _value_ ):
  6. Let _p_ be the Record {[[Key]]: _key_, [[Value]]: _value_}.
  7. Append _p_ as the last element of _entries_.
features: [Symbol, WeakMap, symbols-as-weakmap-keys]
---*/

var sym = Symbol('a description');
var results = [];
var set = WeakMap.prototype.set;
WeakMap.prototype.set = function(key, value) {
  results.push({
    _this: this,
    key: key,
    value: value
  });
  return set.call(this, key, value);
};
var map = new WeakMap([
  [sym, 42],
  [Symbol.hasInstance, 43],
]);

assert.sameValue(results.length, 2, 'Called set() for each entry');
assert.sameValue(results[0].key, sym, 'Adds object in order - first key, regular symbol');
assert.sameValue(results[0].value, 42, 'Adds object in order - first value');
assert.sameValue(results[0]._this, map, 'Adds object in order - this');
assert.sameValue(results[1].key, Symbol.hasInstance, 'Adds object in order - second key, well-known symbol');
assert.sameValue(results[1].value, 43, 'Adds object in order - second value');
assert.sameValue(results[1]._this, map, 'Adds object in order - this');
