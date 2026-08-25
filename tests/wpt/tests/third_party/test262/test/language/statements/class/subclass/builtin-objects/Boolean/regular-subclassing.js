// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.3.1
description: Subclassing Function
info: |
  19.3.1 The Boolean Constructor

  The Boolean constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition.
  ...
---*/

class Bln extends Boolean {}

var b1 = new Bln(1);

assert.notSameValue(b1, true, 'b1 is an Boolean object');
assert.sameValue(b1.valueOf(), true);

var b2 = new Bln(0);
assert.notSameValue(b2, false, 'bln is an Boolean object');
assert.sameValue(b2.valueOf(), false);
