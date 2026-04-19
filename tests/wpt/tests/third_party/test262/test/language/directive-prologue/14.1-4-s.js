// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 14.1-4-s
description: >
    'use strict' directive - not recognized if contains Line
    Continuation
flags: [noStrict]
---*/

  function foo()
  {
    'use str\
ict';
     return (this !== undefined);
  }

assert(foo.call(undefined));
