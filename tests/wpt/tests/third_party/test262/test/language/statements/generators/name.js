// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Assignment of function `name` attribute
es6id: 14.4.12
info: |
    GeneratorDeclaration :
        function * BindingIdentifier ( FormalParameters ) { GeneratorBody }

    [...]
    6. Perform SetFunctionName(F, name).
includes: [propertyHelper.js]
features: [generators]
---*/

function* g() {}

verifyProperty(g, "name", {
  value: "g",
  writable: false,
  enumerable: false,
  configurable: true,
});
