// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2.1
description: Subclassing GeneratorFunction
info: |
  25.2.1 The GeneratorFunction Constructor

  ...

  GeneratorFunction is designed to be subclassable. It may be used as the value
  of an extends clause of a class definition. Subclass constructors that intend
  to inherit the specified GeneratorFunction behaviour must include a super call
  to the GeneratorFunction constructor to create and initialize subclass
  instances with the internal slots necessary for built-in GeneratorFunction
  behaviour.
  ...
---*/

var GeneratorFunction = Object.getPrototypeOf(function* () {}).constructor;

class Gfn extends GeneratorFunction {}

var gfn = new Gfn('a', 'yield a; yield a * 2;');

var iter = gfn(42);

assert.sameValue(iter.next().value, 42);
assert.sameValue(iter.next().value, 84);
