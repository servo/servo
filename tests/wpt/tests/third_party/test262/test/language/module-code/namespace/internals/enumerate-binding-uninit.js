// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-enumerate-object-properties
description: >
  Test for-in enumeration with uninitialized binding.
info: |
  13.7.5.15 EnumerateObjectProperties (O)
    ...
    EnumerateObjectProperties must obtain the own property keys of the
    target object by calling its [[OwnPropertyKeys]] internal method.
    Property attributes of the target object must be obtained by
    calling its [[GetOwnProperty]] internal method.

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

import* as self from "./enumerate-binding-uninit.js";

assert.throws(ReferenceError, function() {
  for (var key in self) {
    throw new Test262Error();
  }
});

export default 0;
