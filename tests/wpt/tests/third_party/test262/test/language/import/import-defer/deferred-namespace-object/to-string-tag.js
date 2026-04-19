// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-getmodulenamespace
description: >
  Deferred namespace objects have a "Deferred Module" @@toStringTag
info: |
  ModuleNamespaceCreate ( _module_, _exports_, _phase_ )
    1. ...
    1. Let _M_ be MakeBasicObject(_internalSlotsList_).
    1. ...
    1. If _phase_ is ~defer~, then
      1. ...
      1. Let _toStringTag_ be *"Deferred Module"*.
    1. Else,
      1. ...
    1. Create an own data property of _M_ named %Symbol.toStringTag% whose [[Value]]
       is _toStringTag_ and whose [[Writable]], [[Enumerable]], and [[Configurable]]
       attributes are false.
    1. Return _M_.

flags: [module]
features: [import-defer]
includes: [propertyHelper.js]
---*/

import defer * as ns from "./dep_FIXTURE.js";

verifyProperty(ns, Symbol.toStringTag, {
  value: "Deferred Module",
  writable: false,
  enumerable: false,
  configurable: false,
});
