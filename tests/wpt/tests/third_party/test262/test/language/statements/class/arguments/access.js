// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class arguments access
---*/
var constructCounts = {
  base: 0,
  subclass: 0,
  subclass2: 0
};

class Base {
  constructor() {
    constructCounts.base++;
    assert.sameValue(arguments.length, 2, "The value of `arguments.length` is `2`");
    assert.sameValue(arguments[0], 1, "The value of `arguments[0]` is `1`");
    assert.sameValue(arguments[1], 2, "The value of `arguments[1]` is `2`");
  }
}

var b = new Base(1, 2);

class Subclass extends Base {
  constructor() {
    constructCounts.subclass++;
    assert.sameValue(arguments.length, 2, "The value of `arguments.length` is `2`");
    assert.sameValue(arguments[0], 3, "The value of `arguments[0]` is `3`");
    assert.sameValue(arguments[1], 4, "The value of `arguments[1]` is `4`");
    super(1, 2);
  }
}

var s = new Subclass(3, 4);
assert.sameValue(Subclass.length, 0, "The value of `Subclass.length` is `0`, because there are 0 formal parameters");

class Subclass2 extends Base {
  constructor(x, y) {
    constructCounts.subclass2++;
    assert.sameValue(arguments.length, 2, "The value of `arguments.length` is `2`");
    assert.sameValue(arguments[0], 3, "The value of `arguments[0]` is `3`");
    assert.sameValue(arguments[1], 4, "The value of `arguments[1]` is `4`");
    super(1, 2);
  }
}

var s2 = new Subclass2(3, 4);
assert.sameValue(Subclass2.length, 2, "The value of `Subclass2.length` is `2`, because there are 2 formal parameters");


assert.sameValue(constructCounts.base, 3, "The value of `constructCounts.base` is `3`");
assert.sameValue(constructCounts.subclass, 1, "The value of `constructCounts.subclass` is `1`");
assert.sameValue(constructCounts.subclass2, 1, "The value of `constructCounts.subclass2` is `1`");
