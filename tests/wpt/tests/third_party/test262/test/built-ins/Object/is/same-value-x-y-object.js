// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.10
description: >
    Object.is ( value1, value2 )

    ...
    10. Return true if x and y are the same Object value. Otherwise, return false.
---*/

var a = {};
var b = Object(0);
var c = new Object("");
var d = [];
var e = Array();
var f = new Array();

assert.sameValue(Object.is(a, a), true, "`Object.is(a, a)` returns `true`");
assert.sameValue(Object.is(b, b), true, "`Object.is(b, b)` returns `true`");
assert.sameValue(Object.is(c, c), true, "`Object.is(c, c)` returns `true`");
assert.sameValue(Object.is(d, d), true, "`Object.is(d, d)` returns `true`");
assert.sameValue(Object.is(e, e), true, "`Object.is(e, e)` returns `true`");
assert.sameValue(Object.is(f, f), true, "`Object.is(f, f)` returns `true`");
