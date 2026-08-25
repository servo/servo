// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class constructor
---*/
var count = 0;
class C {
  constructor() {
    assert.sameValue(
      Object.getPrototypeOf(this),
      C.prototype,
      "`Object.getPrototypeOf(this)` returns `C.prototype`"
    );
    count++;
  }
}
assert.sameValue(
  C,
  C.prototype.constructor,
  "The value of `C` is `C.prototype.constructor`"
);
var desc = Object.getOwnPropertyDescriptor(C.prototype, 'constructor');
assert.sameValue(desc.configurable, true, "The value of `desc.configurable` is `true`, after executing `var desc = Object.getOwnPropertyDescriptor(C.prototype, 'constructor');`");
assert.sameValue(desc.enumerable, false, "The value of `desc.enumerable` is `false`, after executing `var desc = Object.getOwnPropertyDescriptor(C.prototype, 'constructor');`");
assert.sameValue(desc.writable, true, "The value of `descr.writable` is `true`, after executing `var desc = Object.getOwnPropertyDescriptor(C.prototype, 'constructor');`");

var c = new C();
assert.sameValue(count, 1, "The value of `count` is `1`");
assert.sameValue(
  Object.getPrototypeOf(c),
  C.prototype,
  "`Object.getPrototypeOf(c)` returns `C.prototype`"
);
