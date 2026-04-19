// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 14.1-14-s
description: semicolon insertion may come before 'use strict' directive
flags: [noStrict]
---*/

  function foo()
  {
    "another directive"
    "use strict" ;
    return (this === undefined);
  }

assert(foo.call(undefined));
