// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10-0-12
description: with introduces scope - name lookup finds property
flags: [noStrict]
---*/

  function f(o) {

    function innerf(o) {
      with (o) {
        return x;
      }
    }

    return innerf(o);
  }

assert.sameValue(f({x:42}), 42);
