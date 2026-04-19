// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.values
description: Object.values does not see inherited properties.
author: Jordan Harband
---*/

var F = function G() {};
F.prototype.a = {};
F.prototype.b = {};

var f = new F();
f.b = {}; // shadow the prototype
f.c = {}; // solely an own property

var result = Object.values(f);

assert.sameValue(Array.isArray(result), true, 'result is an array');
assert.sameValue(result.length, 2, 'result has 2 items');

assert.sameValue(result[0], f.b, 'first value is f.b');
assert.sameValue(result[1], f.c, 'second value is f.c');
