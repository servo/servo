// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: static class fields forbid PropName 'prototype' (no early error -- PropName of ComputedPropertyName)
esid: sec-class-definitions-static-semantics-early-errors
features: [class, class-static-fields-public]
info: |
  14.6.13 Runtime Semantics: ClassDefinitionEvaluation

  ...
  16. Perform MakeConstructor(F, false, proto).
  ...

  9.2.10 MakeConstructor ( F [ , writablePrototype [ , prototype ] ] )

  6. Perform ! DefinePropertyOrThrow(F, "prototype", PropertyDescriptor { [[Value]]: prototype,
    [[Writable]]: writablePrototype, [[Enumerable]]: false, [[Configurable]]: false }).
---*/

var x = "prototype";

assert.throws(TypeError, function() {
  class C {
    static [x] = 42;
  }
});

assert.throws(TypeError, function() {
  class C {
    static [x];
  }
});
