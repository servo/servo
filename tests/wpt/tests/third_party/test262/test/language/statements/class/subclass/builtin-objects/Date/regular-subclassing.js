// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 20.3.2
description: Subclassing the String object
info: |
  20.3.2 The Date Constructor

  ...

  The Date constructor is a single function whose behaviour is overloaded based
  upon the number and types of its arguments.

  The Date constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified Date behaviour must include a super call to
  the Date constructor to create and initialize the subclass instance with a
  [[DateValue]] internal slot.
---*/

class D extends Date {}

var d1 = new D(1859, '10', 24, 11);
assert.sameValue(d1.getFullYear(), 1859);
assert.sameValue(d1.getMonth(), 10);
assert.sameValue(d1.getDate(), 24);

var d2 = new D(-3474558000000);
assert.sameValue(d2.getUTCFullYear(), 1859);
assert.sameValue(d2.getUTCMonth(), 10);
assert.sameValue(d2.getUTCDate(), 24);

var d3 = new D();
var d4 = new Date();
assert.sameValue(d3.getFullYear(), d4.getFullYear());
assert.sameValue(d3.getMonth(), d4.getMonth());
assert.sameValue(d3.getDate(), d4.getDate());
