// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2
description: >
    No need for parentheses even for lower-precedence expression body
---*/

var square = x => x * x;
assert.sameValue(square(3), 9);
