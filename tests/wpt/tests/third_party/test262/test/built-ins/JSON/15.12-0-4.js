// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    This test should be run without any built-ins being added/augmented.
    The last paragraph in section 15 says "every other property described
    in this section has the attribute {... [[Enumerable]] : false ...}
    unless otherwise specified. This default applies to the properties on
    JSON, and we should not be able to enumerate them.
es5id: 15.12-0-4
description: JSON object's properties must be non enumerable
---*/

var o = JSON;
var i = 0;
for (var p in o) {
  i++;
}


assert.sameValue(i, 0, 'i');
