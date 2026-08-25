// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5.3
description: >
    computed property getter names can be called "constructor"
---*/
class C4 {
  get ['constructor']() {}
}
