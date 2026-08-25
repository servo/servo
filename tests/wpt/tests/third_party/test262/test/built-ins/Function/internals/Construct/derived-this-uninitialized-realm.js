// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ecmascript-function-objects-construct-argumentslist-newtarget
description: >
  Error when derived constructor does not initialize the `this` binding
  (honoring the Realm of the current execution context)
info: |
  [...]
  15. Return ? envRec.GetThisBinding().

  8.1.1.3.4 GetThisBinding ()

  [...]
  3. If envRec.[[ThisBindingStatus]] is "uninitialized", throw a ReferenceError
     exception.
features: [cross-realm, class]
---*/

var C = $262.createRealm().global.eval(
  '(class C extends Object {' +
  '  constructor() {}' +
  '});'
);

assert.throws(ReferenceError, function() {
  new C();
});
