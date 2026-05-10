// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10-2-3
description: with - expression being string
flags: [noStrict]
---*/

  var o = "str";
  var foo = 1;
    with (o) {
      foo = 42;
    }

assert.sameValue(foo, 42);
