// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    computed property names can be "constructor"
---*/
class C {
  ['constructor']() {}
  ['constructor']() {}
}
