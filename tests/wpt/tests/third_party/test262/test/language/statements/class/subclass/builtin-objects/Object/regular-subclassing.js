// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.1
description: Subclassing Object
info: |
  19.1.1 The Object Constructor

  The Object constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition.
---*/

class Obj extends Object {}

var obj = new Obj();

assert.notSameValue(
  Object.getPrototypeOf(obj), Object.prototype,
  'returns the class prototype'
);
