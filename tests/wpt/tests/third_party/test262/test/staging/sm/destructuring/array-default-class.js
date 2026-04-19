// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Function in computed property in class expression in array destructuring default
info: bugzilla.mozilla.org/show_bug.cgi?id=1322314
esid: pending
---*/

function* g([
  a = class E {
    [ (function() { return "foo"; })() ]() {
      return 10;
    }
  }
]) {
  yield a;
}

let C = [...g([])][0];
let x = new C();
assert.sameValue(x.foo(), 10);

C = [...g([undefined])][0];
x = new C();
assert.sameValue(x.foo(), 10);
