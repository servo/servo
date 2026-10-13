// Copyright (C) 2026 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-modulenamespacecreate
description: >
  Deferred namespaces of JSON modules have a "Deferred Module" @@toStringTag
  and expose the parsed JSON value as their "default" export
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

  ParseJSONModule ( _source_ )
    1. Let _json_ be ? Call(%JSON.parse%, *undefined*, « _source_ »).
    1. Return CreateDefaultExportSyntheticModule(_json_).
flags: [module]
features: [import-defer, import-attributes, json-modules]
includes: [propertyHelper.js]
---*/

import defer * as ns from "./json-module_FIXTURE.json" with { type: "json" };

verifyProperty(ns, Symbol.toStringTag, {
  value: "Deferred Module",
  writable: false,
  enumerable: false,
  configurable: false,
});

assert.sameValue(typeof ns.default, "object", "The default export is the parsed JSON object");
assert.sameValue(Object.getPrototypeOf(ns.default), Object.prototype);
assert.sameValue(ns.default.test262, "JSON module");
assert.sameValue(ns.default.number, 42);
