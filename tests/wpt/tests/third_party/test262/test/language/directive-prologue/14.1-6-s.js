// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 14.1-6-s
description: >
    'use strict' directive - not recognized if contains a <TAB>
    instead of a space
flags: [noStrict]
---*/

  function foo()
  {
    'use	strict';
     return (this !== undefined);
  }

assert(foo.call(undefined));
