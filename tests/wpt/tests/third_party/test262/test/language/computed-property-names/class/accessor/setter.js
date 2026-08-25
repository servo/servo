// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    Computed property names for setters
---*/
var calls = 0;
class C {
  set ['a'](_) {
    calls++;
  }
}
new C().a = 'A';
assert.sameValue(calls, 1, "The value of `calls` is `1`, after executing `new C().a = 'A';`");
