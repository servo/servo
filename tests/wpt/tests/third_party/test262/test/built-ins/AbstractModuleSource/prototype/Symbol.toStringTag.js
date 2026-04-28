// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%abstractmodulesource%25.prototype.@@tostringtag
description: >
    %AbstractModuleSource%.prototype[@@toStringTag] property descriptor
info: |
    28.3.3.2 get %AbstractModuleSource%.prototype [ @@toStringTag ]

    1. Let O be the this value.
    2. If O is not an Object, return undefined.
    3. If O does not have a [[ModuleSourceClassName]] internal slot, return undefined.
    4. Let name be O.[[ModuleSourceClassName]].
    5. Assert: name is a String.
    6. Return name.

    This property has the attributes { [[Enumerable]]: false, [[Configurable]]: true }.
flags: [module]
features: [source-phase-imports]
includes: [propertyHelper.js]
---*/

assert.sameValue(typeof $262.AbstractModuleSource, 'function');
verifyProperty($262.AbstractModuleSource.prototype, Symbol.toStringTag, {
  enumerable: false,
  configurable: true,
  set: undefined,
  value: undefined,
}, {
  restore: true,
});

// Return undefined if this value does not have a [[ModuleSourceClassName]] internal slot.
const ToStringTag = Object.getOwnPropertyDescriptor($262.AbstractModuleSource.prototype, Symbol.toStringTag).get;
assert.sameValue(typeof ToStringTag, 'function');
assert.sameValue(ToStringTag.call(262), undefined);
assert.sameValue(ToStringTag.call($262.AbstractModuleSource.prototype), undefined);
