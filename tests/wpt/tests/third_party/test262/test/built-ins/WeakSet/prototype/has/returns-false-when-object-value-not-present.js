// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.has
description: >
  Return false when an Object value is not present in the WeakSet entries.
info: |
  WeakSet.prototype.has ( _value_ )
  6. Return *false*.
features: [WeakSet]
---*/

var foo = {};
var bar = {};
var s = new WeakSet();

assert.sameValue(s.has(foo), false);

s.add(foo);
assert.sameValue(s.has(bar), false);

s.delete(foo);
assert.sameValue(s.has(foo), false);
