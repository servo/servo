// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2
description: >
    Expression Body implicit return
---*/
var plusOne = v => v + 1;
assert.sameValue(plusOne(1), 2);
