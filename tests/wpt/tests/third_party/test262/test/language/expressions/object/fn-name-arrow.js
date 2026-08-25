// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 12.2.6.9
description: Assignment of function `name` attribute (ArrowFunction)
info: |
    6. If IsAnonymousFunctionDefinition(AssignmentExpression) is true, then
       a. Let hasNameProperty be HasOwnProperty(propValue, "name").
       b. ReturnIfAbrupt(hasNameProperty).
       c. If hasNameProperty is false, perform SetFunctionName(propValue,
          propKey).
includes: [propertyHelper.js]
features: [Symbol]
---*/

var namedSym = Symbol('test262');
var anonSym = Symbol();
var o;

o = {
  id: () => {},
  [anonSym]: () => {},
  [namedSym]: () => {}
};

verifyProperty(o.id, 'name', {
  value: 'id',
  writable: false,
  enumerable: false,
  configurable: true,
});

verifyProperty(o[anonSym], 'name', {
  value: '',
  writable: false,
  enumerable: false,
  configurable: true,
});

verifyProperty(o[namedSym], 'name', {
  value: '[test262]',
  writable: false,
  enumerable: false,
  configurable: true,
});
