// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10-2-1
description: with - expression being Number
flags: [noStrict]
---*/

  var o = 2;
  var foo = 1;
    with (o) {
      foo = 42;
    }

assert.sameValue(foo, 42);
