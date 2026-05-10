// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.has
description: >
    Set.prototype.has ( value )

    ...
    5. Repeat for each e that is an element of entries,
      a. If e is not empty and SameValueZero(e, value) is true, return true.
    ...

features: [Symbol]
---*/

var s = new Set();
var a = Symbol();

s.add(a)

assert.sameValue(s.has(a), true, "`s.has(a)` returns `true`");
