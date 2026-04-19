// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2
description: >
    Parenthesize the body to return an object literal expression
---*/

var keyMaker = val => ({ key: val });
assert.sameValue(keyMaker(1).key, 1);
