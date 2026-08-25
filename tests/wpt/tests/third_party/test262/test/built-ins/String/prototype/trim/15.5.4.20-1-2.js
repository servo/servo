// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-1-2
description: String.prototype.trim throws TypeError when string is null
---*/


assert.throws(TypeError, function() {
  String.prototype.trim.call(null);
});
