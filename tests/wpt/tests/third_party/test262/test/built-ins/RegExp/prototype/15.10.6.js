// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.10.6
description: RegExp.prototype is itself a not RegExp
---*/

  var s = Object.prototype.toString.call(RegExp.prototype);

assert.sameValue(s, '[object Object]', 's');
