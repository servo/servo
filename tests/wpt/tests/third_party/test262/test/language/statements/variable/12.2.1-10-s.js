// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.2.1-10-s
description: "Strict Mode: an indirect eval assigning into 'eval' does not throw"
---*/

  var s = eval;
  s('eval = 42;');
