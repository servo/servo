// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.2.1-20-s
description: >
    Strict Mode: an indirect eval declaring a var named 'arguments'
    does not throw
---*/

  var s = eval;
  s('var arguments;');
