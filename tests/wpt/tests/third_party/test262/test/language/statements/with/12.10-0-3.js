// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10-0-3
description: with introduces scope - that is captured by function expression
flags: [noStrict]
---*/

  var o = {prop: "12.10-0-3 before"};
  var f;

  with (o) {
    f = function () { return prop; }
  }
  o.prop = "12.10-0-3 after";

assert.sameValue(f(), "12.10-0-3 after", 'f()');
