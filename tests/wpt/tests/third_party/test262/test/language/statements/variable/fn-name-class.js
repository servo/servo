// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 13.3.2.4
description: Assignment of function `name` attribute (ClassExpression)
info: |
    VariableDeclaration : BindingIdentifier Initializer

    [...]
    7. If IsAnonymousFunctionDefinition(Initializer) is true, then
       a. Let hasNameProperty be HasOwnProperty(value, "name").
       b. ReturnIfAbrupt(hasNameProperty).
       c. If hasNameProperty is false, perform SetFunctionName(value,
          bindingId).
includes: [propertyHelper.js]
features: [class]
---*/

var xCls = class x {};
var cls = class {};
var xCls2 = class { static name() {} };

assert.notSameValue(xCls.name, 'xCls');
assert.notSameValue(xCls2.name, 'xCls2');

verifyProperty(cls, 'name', {
  value: 'cls',
  writable: false,
  enumerable: false,
  configurable: true,
});
