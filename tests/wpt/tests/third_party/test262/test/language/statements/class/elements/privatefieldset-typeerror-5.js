// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: TypeError when setting private field not in `this`'s [[PrivateFieldValues]]
esid: sec-putvalue
info: |
  PutValue ( V, W )
    ...
    6. Else if IsPropertyReference(V), then
      ...
      b. If IsPrivateReference(V), then
        i. Let env be the running execution context's PrivateNameEnvironment.
        ii. Let field be ? ResolveBinding(GetReferencedName(V), env).
        iii. Assert: field is a Private Name.
        iv. Perform ? PrivateFieldSet(field, base, W).

  PrivateFieldSet (P, O, value )
    1. Assert: P is a Private Name value.
    2. If O is not an object, throw a TypeError exception.
    3. Let entry be PrivateFieldFind(P, O).
    4. If entry is empty, throw a TypeError exception.

  PrivateFieldFind (P, O)
    1. Assert: P is a Private Name value.
    2. Assert: O is an object with a [[PrivateFieldValues]] internal slot.
    3. For each element entry in O.[[PrivateFieldValues]],
      a. If entry.[[PrivateName]] is P, return entry.
    4. Return empty.

features: [class, class-fields-private]
---*/


class Outer {
  #x = 42;

  innerclass() {
    var self = this;

    return class extends Outer {
      #x = 'not42';
      f() {
        self.#x = 1;
      }
    }
  }
}

var Inner = new Outer().innerclass();
var i = new Inner();

assert.throws(TypeError, function() {
  // when f() is called, the entry in the PrivateNameEnironment's environment
  // record for '#x' will contain the Inner class's Private Name for '#x'.
  // When this Private Name is used for lookup on the `self` object, it
  // will not be found (as the `self` object has the Outer's Private Name for #x)
  i.f();
})
