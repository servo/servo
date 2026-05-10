// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 14.1-7-s
description: "'use strict' directive - not recognized if upper case"
flags: [noStrict]
---*/

  function foo()
  {
    'Use Strict';
     return (this !== undefined);
  }

assert(foo.call(undefined));
