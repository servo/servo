// Copyright 2019 Google, LLC.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional call invoked on super method should be equivalent to call
info: |
  OptionalExpression
    MemberExpression OptionalChain
      SuperProperty OptionalChain
features: [optional-chaining]
---*/

let called = false;
let context;
class Base {
    method() {
      called = true;
      context = this;
    }
}
class Foo extends Base {
    method() {
      super.method?.();
    }
}
const foo = new Foo();
foo.method();
assert(foo === context);
assert.sameValue(called, true);
