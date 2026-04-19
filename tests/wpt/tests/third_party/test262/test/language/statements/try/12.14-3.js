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
es5id: 12.14-3
description: >
    catch doesn't change declaration scope - var declaration are
    visible outside when name different from catch parameter
---*/

  try {
    throw new Error();
  }
  catch (e) {
    var foo = "declaration in catch";
  }

assert.sameValue(foo, "declaration in catch");
