// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.4
description: >
  Delete property.
info: |
  26.1.4 Reflect.deleteProperty ( target, propertyKey )

  ...
  4. Return target.[[Delete]](key).
features: [Reflect]
---*/

var o = {
  prop: 42
};

Reflect.deleteProperty(o, 'prop');

assert.sameValue(o.hasOwnProperty('prop'), false);
