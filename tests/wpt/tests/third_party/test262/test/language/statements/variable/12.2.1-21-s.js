// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.2.1-21-s
description: >
    Strict Mode: an indirect eval assigning into 'arguments' does not
    throw
---*/

  var s = eval;
  s('arguments = 42;');
