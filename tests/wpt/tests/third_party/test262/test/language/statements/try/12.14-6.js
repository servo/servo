// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    local vars must not be visible outside with block
    local functions must not be visible outside with block
    local function expresssions should not be visible outside with block
    local vars must shadow outer vars
    local functions must shadow outer functions
    local function expresssions must shadow outer function expressions
    eval should use the appended object to the scope chain
es5id: 12.14-6
description: >
    catch introduces scope - block-local function expression must
    shadow outer function expression
---*/

  var o = {foo : function () { return 42;}};

  try {
    throw o;
  }
  catch (e) {
    var foo = function () {};
  }

assert.sameValue(foo(), undefined);
