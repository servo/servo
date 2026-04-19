// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2
description: >
    Statement body needs braces, must use 'return' explicitly if not void
---*/
var plusOne = v => {
  return v + 1;
};

assert.sameValue(plusOne(1), 2);
