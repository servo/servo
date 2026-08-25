// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 13.3.2.4
description: Assignment of function `name` attribute (GeneratorExpression)
info: |
    VariableDeclaration : BindingIdentifier Initializer

    [...]
    7. If IsAnonymousFunctionDefinition(Initializer) is true, then
       a. Let hasNameProperty be HasOwnProperty(value, "name").
       b. ReturnIfAbrupt(hasNameProperty).
       c. If hasNameProperty is false, perform SetFunctionName(value,
          bindingId).
includes: [propertyHelper.js]
features: [generators]
---*/

var xGen = function* x() {};
var gen = function*() {};

assert(xGen.name !== 'xGen');

verifyProperty(gen, 'name', {
  value: 'gen',
  writable: false,
  enumerable: false,
  configurable: true,
});
