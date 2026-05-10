// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.propertyisenumerable
description: >
  Test Object.prototype.propertyIsEnumerable() with uninitialized binding.
info: |
  19.1.3.4 Object.prototype.propertyIsEnumerable ( V )
    ...
    3. Let desc be ? O.[[GetOwnProperty]](P).
    ...

  9.4.6.4 [[GetOwnProperty]] (P)
    ...
    4. Let value be ? O.[[Get]](P, O).
    ...

  9.4.6.7 [[Get]] (P, Receiver)
    ...
    12. Let targetEnvRec be targetEnv's EnvironmentRecord.
    13. Return ? targetEnvRec.GetBindingValue(binding.[[BindingName]], true).

  8.1.1.1.6 GetBindingValue ( N, S )
    ...
    If the binding for N in envRec is an uninitialized binding, throw a ReferenceError exception.
    ...

flags: [module]
---*/

import* as self from "./object-propertyIsEnumerable-binding-uninit.js";

assert.throws(ReferenceError, function() {
  Object.prototype.propertyIsEnumerable.call(self, "default");
});

export default 0;
