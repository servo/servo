// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 14.3.9
description: Assignment of function `name` attribute ("get" accessor)
info: |
    MethodDefinition : get PropertyName ( ) { FunctionBody }

    [...]
    8. Perform SetFunctionName(closure, propKey, "get").
includes: [propertyHelper.js]
features: [Symbol]
---*/

var namedSym = Symbol('test262');
var anonSym = Symbol();
var o, getter;

o = {
  get id() {},
  get [anonSym]() {},
  get [namedSym]() {}
};

getter = Object.getOwnPropertyDescriptor(o, 'id').get;
verifyProperty(getter, 'name', {
  value: 'get id',
  writable: false,
  enumerable: false,
  configurable: true,
});

getter = Object.getOwnPropertyDescriptor(o, anonSym).get;
verifyProperty(getter, 'name', {
  value: 'get ',
  writable: false,
  enumerable: false,
  configurable: true,
});

getter = Object.getOwnPropertyDescriptor(o, namedSym).get;
verifyProperty(getter, 'name', {
  value: 'get [test262]',
  writable: false,
  enumerable: false,
  configurable: true,
});
