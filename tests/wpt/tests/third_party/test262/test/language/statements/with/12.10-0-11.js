// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10-0-11
description: with introduces scope - name lookup finds inner variable
flags: [noStrict]
---*/

  function f(o) {

    function innerf(o) {
      var x = 42;

      with (o) {
        return x;
      }
    }

    return innerf(o);
  }

assert.sameValue(f({}), 42);
