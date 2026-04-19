// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.clear
description: >
  Clears a Map.
info: |
  Map.prototype.clear ( )

  ...
  4. Let entries be the List that is the value of M’s [[MapData]] internal slot.
  5. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    a. Set p.[[key]] to empty.
    b. Set p.[[value]] to empty.
  6. Return undefined.
features: [Symbol]
---*/

var m1 = new Map([
  ['foo', 'bar'],
  [1, 1]
]);
var m2 = new Map();
var m3 = new Map();
m2.set('foo', 'bar');
m2.set(1, 1);
m2.set(Symbol('a'), Symbol('a'));

m1.clear();
m2.clear();
m3.clear();

assert.sameValue(m1.size, 0);
assert.sameValue(m2.size, 0);
assert.sameValue(m3.size, 0);
