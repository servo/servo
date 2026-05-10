// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    In a class, duplicate computed property setter names produce only a single property of
    that name, whose value is the value of the last property of that name.
---*/
var calls = 0;
class C {
  set ['a'](_) {
    calls++;
  }
}
new C().a = 'A';
assert.sameValue(calls, 1, "The value of `calls` is `1`, after executing `new C().a = 'A';`");

calls = 0;
class C2 {
  set b(_) {
    throw new Test262Error("The first `b` setter definition in `C2` is unreachable");
  }
  set ['b'](_) {
    calls++;
  }
}
new C2().b = 'B';
assert.sameValue(calls, 1, "The value of `calls` is `1`, after executing `new C2().b = 'B';`");

calls = 0;
class C3 {
  set c(_) {
    throw new Test262Error("The first `c` setter definition in `C3` is unreachable");
  }
  set ['c'](_) {
    throw new Test262Error("The second `c` setter definition in `C3` is unreachable");
  }
  set ['c'](_) {
    calls++
  }
}
new C3().c = 'C';
assert.sameValue(calls, 1, "The value of `calls` is `1`, after executing `new C3().c = 'C';`");

calls = 0;
class C4 {
  set ['d'](_) {
    throw new Test262Error("The first `d` setter definition in `C4` is unreachable");
  }
  set d(_) {
    calls++
  }
}
new C4().d = 'D';
assert.sameValue(calls, 1, "The value of `calls` is `1`, after executing `new C4().d = 'D';`");
