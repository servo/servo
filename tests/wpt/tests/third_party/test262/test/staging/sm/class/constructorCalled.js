// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// The constructor specified should get called, regardless of order, or
// other distractions

var called = false;
class a { constructor(x) { assert.sameValue(x, 4); called = true } }
new a(4);
assert.sameValue(called, true);

called = false;
var aExpr = class { constructor(x) { assert.sameValue(x, 4); called = true } };
new aExpr(4);
assert.sameValue(called, true);

called = false;
class b { constructor() { called = true } method() { } }
new b();
assert.sameValue(called, true);

called = false;
var bExpr = class { constructor() { called = true } method() { } };
new bExpr();
assert.sameValue(called, true);

called = false;
class c { method() { } constructor() { called = true; } }
new c();
assert.sameValue(called, true);

called = false;
var cExpr = class { method() { } constructor() { called = true; } }
new cExpr();
assert.sameValue(called, true);

called = false;
class d { ["constructor"]() { throw new Error("NO"); } constructor() { called = true; } }
new d();
assert.sameValue(called, true);

called = false;
var dExpr = class { ["constructor"]() { throw new Error("NO"); } constructor() { called = true; } }
new dExpr();
assert.sameValue(called, true);

