// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 13.3.1.4
description: Assignment of function `name` attribute (ArrowFunction)
info: |
    LexicalBinding : BindingIdentifier Initializer

    [...]
    6. If IsAnonymousFunctionDefinition(Initializer) is true, then
       a. Let hasNameProperty be HasOwnProperty(value, "name").
       b. ReturnIfAbrupt(hasNameProperty).
       c. If hasNameProperty is false, perform SetFunctionName(value,
          bindingId).
includes: [propertyHelper.js]
---*/

let arrow = () => {};

verifyProperty(arrow, 'name', {
  value: 'arrow',
  writable: false,
  enumerable: false,
  configurable: true,
});
