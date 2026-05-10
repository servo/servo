// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.1.1
description: Subclassing the Map object
info: |
  23.1.1 The Map Constructor

  ...

  The Map constructor is designed to be subclassable. It may be used as the
  value in an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified Map behaviour must include a super call to the
  Map constructor to create and initialize the subclass instance with the
  internal state necessary to support the Map.prototype built-in methods.
---*/

class M extends Map {}

var map = new M([{ 'foo': 'bar' }]);

assert.sameValue(map.size, 1);

map.set('bar', 'baz');

assert.sameValue(map.size, 2);
