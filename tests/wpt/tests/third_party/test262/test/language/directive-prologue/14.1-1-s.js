// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 14.1-1-s
description: "'use strict' directive - correct usage"
flags: [noStrict]
---*/

  function foo()
  {
    'use strict';
     return(this === undefined);
  }

assert(foo.call(undefined));
