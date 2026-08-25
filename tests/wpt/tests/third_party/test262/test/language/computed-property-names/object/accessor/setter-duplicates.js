// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    In an object, duplicate computed property getter names produce only a single property of
    that name, whose value is the value of the last property of that name.
---*/
var calls = 0;
var A = {
  set ['a'](_) {
    calls++;
  }
};
A.a = 'A';
assert.sameValue(calls, 1, "The value of `calls` is `1`");

calls = 0;
var B = {
  set b(_) {
    throw new Test262Error("The `b` setter definition in `B` is unreachable");
  },
  set ['b'](_) {
    calls++;
  }
};
B.b = 'B';
assert.sameValue(calls, 1, "The value of `calls` is `1`");

calls = 0;
var C = {
  set c(_) {
    throw new Test262Error("The `c` setter definition in `C` is unreachable");
  },
  set ['c'](_) {
    throw new Test262Error("The first `['c']` setter definition in `C` is unreachable");
  },
  set ['c'](_) {
    calls++
  }
};
C.c = 'C';
assert.sameValue(calls, 1, "The value of `calls` is `1`");

calls = 0;
var D = {
  set ['d'](_) {
    throw new Test262Error("The `['d']` setter definition in `D` is unreachable");
  },
  set d(_) {
    calls++
  }
};
D.d = 'D';
assert.sameValue(calls, 1, "The value of `calls` is `1`");
