// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-meta-properties-runtime-semantics-evaluation
description: >
  import.meta is an ordinary object.
info: |
  Runtime Semantics: Evaluation

   ImportMeta : import.meta

    ...
    4. If importMeta is undefined.
        a. Set importMeta to ObjectCreate(null).
        b. Let importMetaValues be ! HostGetImportMetaProperties(module).
        ...
        e. Perform ! HostFinalizeImportMeta(importMeta, module).
        ...
        g. Return importMeta.
    ...
flags: [module]
features: [import.meta]
---*/

// import.meta is an object.
assert.sameValue(typeof import.meta, "object",
                 "typeof import.meta is 'object'");
assert.notSameValue(import.meta, null,
                    "typeof import.meta is 'object' and import.meta isn't |null|.");

assert.throws(TypeError, function() {
    import.meta();
}, "import.meta is not callable");

assert.throws(TypeError, function() {
    new import.meta();
}, "import.meta is not a constructor");

// Note: The properties, the shape of the properties, the extensibility state, and the prototype
//       of import.meta are implementation-defined via HostGetImportMetaProperties and
//       HostFinalizeImportMeta.

// Properties and the prototype can only be modified when import.meta is extensible.
if (Object.isExtensible(import.meta)) {
    assert.sameValue(Object.getOwnPropertyDescriptor(import.meta, "test262prop"), undefined,
                     "test262 test property is not present initially");

    import.meta.test262prop = "blubb";

    assert.sameValue(import.meta.test262prop, "blubb",
                     "Properties can be added and retrieved from import.meta");

    assert.sameValue(delete import.meta.test262prop, true,
                     "Properties can be removed from import.meta");

    assert.sameValue(Object.getOwnPropertyDescriptor(import.meta, "test262prop"), undefined,
                     "test262 test property is no longer present");

    var proto = {};
    Object.setPrototypeOf(import.meta, proto);

    assert.sameValue(Object.getPrototypeOf(import.meta), proto,
                     "[[Prototype]] of import.meta can be changed");
}

Object.preventExtensions(import.meta);
assert.sameValue(Object.isExtensible(import.meta), false,
                 "import.meta is non-extensible after calling |Object.preventExtensions|");

Object.seal(import.meta);
assert.sameValue(Object.isSealed(import.meta), true,
                 "import.meta is sealed after calling |Object.seal|");

Object.freeze(import.meta);
assert.sameValue(Object.isFrozen(import.meta), true,
                 "import.meta is frozen after calling |Object.freeze|");
