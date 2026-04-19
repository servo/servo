// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-10-c-ii-1
description: arguments[i] change with actual parameters
flags: [noStrict]
---*/

function foo(a,b,c)
{
  a = 1; b = 'str'; c = 2.1;
  if(arguments[0] === 1 && arguments[1] === 'str' && arguments[2] === 2.1)
    return true;
}

assert(foo(10,'sss',1), 'foo(10,"sss",1) !== true');
