// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object-initializer
description: Permitted duplicate `__proto__` property (shorthand)
info: |
    Annex B defines an early error for duplicate PropertyName of `__proto__`,
    but this does not apply to properties created from other productions.

    B.3.1 __proto__ Property Names in Object Initializers

    It is a Syntax Error if PropertyNameList of PropertyDefinitionList contains
    any duplicate entries for "__proto__" and at least two of those entries
    were obtained from productions of the form
    PropertyDefinition : PropertyName : AssignmentExpression .
---*/

var __proto__ = 2;
var obj = {
  __proto__,
  __proto__,
};

assert(obj.hasOwnProperty("__proto__"));
assert.sameValue(obj.__proto__, 2);
