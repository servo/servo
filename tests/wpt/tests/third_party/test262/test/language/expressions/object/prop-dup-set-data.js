// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.5_4-4-c-2
description: >
    Object literal - No SyntaxError if a set accessor property definition
    is followed by a data property definition with the same name
---*/

void {
  set foo(x) {},
  foo: 1
};
