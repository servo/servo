// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 14.3.9
description: Assignment of function `name` attribute ("set" accessor)
info: |
    MethodDefinition :
        set PropertyName ( PropertySetParameterList ) { FunctionBody }

    [...]
    7. Perform SetFunctionName(closure, propKey, "set").
includes: [propertyHelper.js]
features: [Symbol]
---*/

var namedSym = Symbol('test262');
var anonSym = Symbol();
var o, setter;

o = {
  set id(_) {},
  set [anonSym](_) {},
  set [namedSym](_) {}
};

setter = Object.getOwnPropertyDescriptor(o, 'id').set;
verifyProperty(setter, 'name', {
  value: 'set id',
  writable: false,
  enumerable: false,
  configurable: true,
});

setter = Object.getOwnPropertyDescriptor(o, anonSym).set;
verifyProperty(setter, 'name', {
  value: 'set ',
  writable: false,
  enumerable: false,
  configurable: true,
});

setter = Object.getOwnPropertyDescriptor(o, namedSym).set;
verifyProperty(setter, 'name', {
  value: 'set [test262]',
  writable: false,
  enumerable: false,
  configurable: true,
});
