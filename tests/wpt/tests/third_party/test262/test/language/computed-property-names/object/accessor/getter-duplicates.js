// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    In an object, duplicate computed property getter names produce only a single property of
    that name, whose value is the value of the last property of that name.
---*/
var A = {
  get ['a']() {
    return 'A';
  }
};
assert.sameValue(A.a, 'A', "The value of `A.a` is `'A'`");

var B = {
  get b() {
    throw new Test262Error("The `b` getter definition in `B` is unreachable");
  },
  get ['b']() {
    return 'B';
  }
};
assert.sameValue(B.b, 'B', "The value of `B.b` is `'B'`");

var C = {
  get c() {
    throw new Test262Error("The `c` getter definition in `C` is unreachable");
  },
  get ['c']() {
    throw new Test262Error("The `['c']` getter definition in `C` is unreachable");
  },
  get ['c']() {
    return 'C';
  }
};
assert.sameValue(C.c, 'C', "The value of `C.c` is `'C'`");

var D = {
  get ['d']() {
    throw new Test262Error("The `['d']` getter definition in `D` is unreachable");
  },
  get d() {
    return 'D';
  }
};
assert.sameValue(D.d, 'D', "The value of `D.d` is `'D'`");
