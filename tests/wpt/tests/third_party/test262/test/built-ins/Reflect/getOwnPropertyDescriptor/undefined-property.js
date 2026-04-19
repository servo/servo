// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.7
description: >
  Return undefined for an undefined property.
info: |
  26.1.7 Reflect.getOwnPropertyDescriptor ( target, propertyKey )

  ...
  4. Let desc be target.[[GetOwnProperty]](key).
  5. ReturnIfAbrupt(desc).
  6. Return FromPropertyDescriptor(desc).

  6.2.4.4 FromPropertyDescriptor ( Desc )

  1. If Desc is undefined, return undefined.
features: [Reflect]
---*/

var result = Reflect.getOwnPropertyDescriptor({}, undefined);
assert.sameValue(result, undefined);
