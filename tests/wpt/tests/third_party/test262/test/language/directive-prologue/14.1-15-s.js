// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 14.1-15-s
description: blank lines may come before 'use strict' directive
flags: [noStrict]
---*/

  function foo()
  {






    "use strict" ;
    return (this === undefined);
  }

assert(foo.call(undefined));
