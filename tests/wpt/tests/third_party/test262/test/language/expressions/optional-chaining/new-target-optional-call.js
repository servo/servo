// Copyright 2019 Google, LLC.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional call invoked on new.target should be equivalent to call
info: |
  OptionalExpression
    MemberExpression OptionalChain
      NewTarget OptionalChain
features: [optional-chaining]
---*/

const newTargetContext = (function() { return this; })();

let called = false;
// should be set to 'undefined' or global context, depending on whether
// mode is strict or sloppy.
let context = null;
function Base() {
  called = true;
  context = this;
}
function Foo(blerg) {
  new.target?.();
}

Reflect.construct(Foo, [], Base);
assert(context === newTargetContext);
assert.sameValue(called, true);
