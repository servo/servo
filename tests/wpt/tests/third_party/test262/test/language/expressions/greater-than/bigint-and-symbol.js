// Copyright (C) 2018 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Relational comparison of BigInt and Symbol values
esid: sec-abstract-relational-comparison
features: [BigInt, Symbol]
---*/
assert.throws(TypeError, function() {
  3n > Symbol('2');
}, '3n > Symbol("2") throws TypeError');

assert.throws(TypeError, function() {
  Symbol('2') > 3n;
}, 'Symbol("2") > 3n throws TypeError');
