// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    In a class, duplicate computed property getter names produce only a single property of
    that name, whose value is the value of the last property of that name.
---*/
class C {
  get ['a']() {
    return 'A';
  }
}
assert.sameValue(new C().a, 'A', "The value of `new C().a` is `'A'`");

class C2 {
  get b() {
    throw new Test262Error("The first `b` getter definition in `C2` is unreachable");
  }
  get ['b']() {
    return 'B';
  }
}
assert.sameValue(new C2().b, 'B', "The value of `new C2().b` is `'B'`");

class C3 {
  get c() {
    throw new Test262Error("The first `c` getter definition in `C3` is unreachable");
  }
  get ['c']() {
    throw new Test262Error("The second `c` getter definition in `C3` is unreachable");
  }
  get ['c']() {
    return 'C';
  }
}
assert.sameValue(new C3().c, 'C', "The value of `new C3().c` is `'C'`");

class C4 {
  get ['d']() {
    throw new Test262Error("The first `d` getter definition in `C4` is unreachable");
  }
  get d() {
    return 'D';
  }
}
assert.sameValue(new C4().d, 'D', "The value of `new C4().d` is `'D'`");
