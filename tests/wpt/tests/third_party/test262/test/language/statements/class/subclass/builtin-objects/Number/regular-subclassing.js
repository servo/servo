// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 20.1.1
description: Subclassing the Number object
info: |
  20.1.1 The Number Constructor

  ...

  The Number constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified Number behaviour must include a super call to
  the Number constructor to create and initialize the subclass instance with a
  [[NumberData]] internal slot.
---*/

class N extends Number {}

var n = new N(42);

assert.sameValue(n.toFixed(2), '42.00');
assert.sameValue(n.toExponential(2), '4.20e+1');
