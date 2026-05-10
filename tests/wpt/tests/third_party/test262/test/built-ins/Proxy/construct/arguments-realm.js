// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
  Arguments array is created in the Realm of the current execution context
info: |
  [...]
  7. Let argArray be CreateArrayFromList(argumentsList).
  8. Let newObj be ? Call(trap, handler, « target, argArray, newTarget »).
  [...]
features: [cross-realm]
---*/

var C = $262.createRealm().global.eval(
  'new Proxy(function() {}, { construct: function(_, args) { return args; } })'
);

assert.sameValue(new C().constructor, Array);
