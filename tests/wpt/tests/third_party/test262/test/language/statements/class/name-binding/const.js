// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class name binding const
---*/
assert.throws(TypeError, function() {
  class C { constructor() { C = 42; } }; new C();
});
assert.throws(TypeError, function() {
  new (class C { constructor() { C = 42; } })
});
assert.throws(TypeError, function() {
  class C { m() { C = 42; } }; new C().m()
});
assert.throws(TypeError, function() {
  new (class C { m() { C = 42; } }).m()
});
assert.throws(TypeError, function() {
  class C { get x() { C = 42; } }; new C().x
});
assert.throws(TypeError, function() {
  (new (class C { get x() { C = 42; } })).x
});
assert.throws(TypeError, function() {
  class C { set x(_) { C = 42; } }; new C().x = 15;
});
assert.throws(TypeError, function() {
  (new (class C { set x(_) { C = 42; } })).x = 15;
});
