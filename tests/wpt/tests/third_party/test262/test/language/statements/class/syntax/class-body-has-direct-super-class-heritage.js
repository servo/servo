// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5.1
description: >
    The opposite of:

      ClassTail : ClassHeritageopt { ClassBody }

      It is a Syntax Error if ClassHeritage is not present and the following algorithm evaluates to true:
        1. Let constructor be ConstructorMethod of ClassBody.
        2. If constructor is empty, return false.
        3. Return HasDirectSuper of constructor.
---*/
class A {}
class B extends A {
  constructor() {
    super();
  }
}


assert.sameValue(typeof B, "function");
