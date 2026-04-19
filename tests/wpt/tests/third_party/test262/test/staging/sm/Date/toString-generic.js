// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Date.prototype.toString is a generic function
info: bugzilla.mozilla.org/show_bug.cgi?id=861219
esid: pending
---*/

// Revised in ECMA 2018, Date.prototype.toString is no longer generic (bug 1381433).

for (var thisValue of [{}, [], /foo/, Date.prototype, new Proxy(new Date(), {})])
  assert.throws(TypeError, () => Date.prototype.toString.call(thisValue));

for (var prim of [null, undefined, 0, 1.2, true, false, "foo", Symbol.iterator])
  assert.throws(TypeError, () => Date.prototype.toString.call(prim));
