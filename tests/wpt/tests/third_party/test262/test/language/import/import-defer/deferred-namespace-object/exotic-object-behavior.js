// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-modulenamespacecreate
description: >
  Deferred namespace objects have the correct MOP implementation
info: |
  ModuleNamespaceCreate ( _module_, _exports_, _phase_ )
    1. Let _internalSlotsList_ be the internal slots listed in <emu-xref href="#table-internal-slots-of-module-namespace-exotic-objects"></emu-xref>.
    1. Let _M_ be MakeBasicObject(_internalSlotsList_).
    1. Set _M_'s essential internal methods to the definitions specified in <emu-xref href="#sec-module-namespace-exotic-objects"></emu-xref>.
    1. ...

  [[GetPrototypeOf]] ( )
    1. Return null.

  [[IsExtensible]] ( )
    1. Return false.

flags: [module]
features: [import-defer]
includes: [propertyHelper.js, compareArray.js]
---*/

import defer * as ns from "./dep_FIXTURE.js";

assert.sameValue(typeof ns, "object", "Deferred namespaces are objects");

assert(!Reflect.isExtensible(ns), "Deferred namespaces are not extensible");
assert.sameValue(Reflect.preventExtensions(ns), true, "Deferred namespaces can made non-extensible");

assert.sameValue(Reflect.getPrototypeOf(ns), null, "Deferred namespaces have a null prototype");
assert.sameValue(Reflect.setPrototypeOf(ns, {}), false, "Deferred namespaces' prototype cannot be changed");
assert.sameValue(Reflect.setPrototypeOf(ns, null), true, "Deferred namespaces' prototype can be 'set' to null");

assert.throws(TypeError, () => Reflect.apply(ns, null, []), "Deferred namespaces are not callable");
assert.throws(TypeError, () => Reflect.construct(ns, [], ns), "Deferred namespaces are not constructable");

assert.compareArray(
  Reflect.ownKeys(ns),
  ["bar", "foo", Symbol.toStringTag],
  "Deferred namespaces' keys are the exports sorted alphabetically, followed by @@toStringTag"
);

// We cannot use `verifyProperty` because the property _looks_ writable, but it's actually not
const desc = Reflect.getOwnPropertyDescriptor(ns, "foo");
assert.sameValue(desc.value, 1, "The value of the 'foo' property is 1");
assert.sameValue(desc.writable, true, "The 'foo' property is writable");
assert.sameValue(desc.enumerable, true, "The 'foo' property is enumerable");
assert.sameValue(desc.configurable, false, "The 'foo' property is not configurable");

assert.sameValue(Reflect.getOwnPropertyDescriptor(ns, "non-existent"), undefined, "No descriptors for non-exports");
