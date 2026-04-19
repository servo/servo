// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    global and block scope const
---*/
const z = 4;

// Block local
{
  const z = 5;
}

assert.sameValue(z, 4);

if (true) {
  const z = 1;
  assert.sameValue(z, 1);
}

