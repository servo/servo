// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    In a class, computed property names for static
    getters can be named "constructor"
---*/
class C {
  static get ['constructor']() {}
}
