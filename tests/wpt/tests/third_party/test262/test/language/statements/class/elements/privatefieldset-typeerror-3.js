// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Referenced lexically scoped private field found in `this`'s [[PrivateFieldValues]]
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

    // create class within in Outerclass -- the PrivateNameEnvironment binding for
    // private field `#x` is resolvable.
    return class extends Outer {
      f() {
        this.#x = 1;
      }
    }
  }

  value() {
    return this.#x;
  }
}

var outer = new Outer();
var Inner = outer.innerclass();
var i = new Inner();

assert.sameValue(outer.value(), 42);
assert.sameValue(i.value(), 42);

i.f();

assert.sameValue(outer.value(), 42, "value is set on inner class instance only");
assert.sameValue(i.value(), 1, "value is set from inner class instance");
