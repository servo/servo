// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-10-c-ii-2-s
description: arguments[i] doesn't map to actual parameters in strict mode
flags: [onlyStrict]
---*/

  function foo(a,b,c)
  {
    arguments[0] = 1; arguments[1] = 'str'; arguments[2] = 2.1;
    return 10 === a && 'sss' === b && 1 === c;
  }

assert(foo(10,'sss',1), 'foo(10,"sss",1) !== true');
