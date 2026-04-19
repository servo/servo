// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-10-c-ii-2
description: arguments[i] map to actual parameter
flags: [noStrict]
---*/

function foo(a,b,c)
{
  arguments[0] = 1; arguments[1] = 'str'; arguments[2] = 2.1;
  if(1 === a && 'str' === b && 2.1 === c)
    return true;
}

assert(foo(10,'sss',1), 'foo(10,"sss",1) !== true');
