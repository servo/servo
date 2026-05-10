// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.keys
description: >
  Test Object.keys() with uninitialized binding.
info: |
  19.1.2.16 Object.keys ( O )
    ...
    2. Let nameList be ? EnumerableOwnProperties(obj, "key").
    ...

  7.3.21 EnumerableOwnProperties ( O, kind )
    ...
    4. For each element key of ownKeys in List order, do
      a. If Type(key) is String, then
        i. Let desc be ? O.[[GetOwnProperty]](key).
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

import* as self from "./object-keys-binding-uninit.js";

assert.throws(ReferenceError, function() {
  Object.keys(self);
});

export default 0;
