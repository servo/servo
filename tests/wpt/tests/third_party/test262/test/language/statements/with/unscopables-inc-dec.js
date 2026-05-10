// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object-environment-records-hasbinding-n
description: >
  @@unscopables should be looked up exactly once for inc/dec.
info: |
  UpdateExpression : LeftHandSideExpression ++
  1. Let lhs be the result of evaluating LeftHandSideExpression.

  GetIdentifierReference ( lex, name, strict )
  [...]
  3. Let exists be ? envRec.HasBinding(name).

  HasBinding ( N )
  [...]
  6. Let unscopables be ? Get(bindings, @@unscopables).
flags: [noStrict]
features: [Symbol.unscopables]
---*/

var unscopablesGetterCalled = 0;
var a, b, flag = true;
with (a = { x: 7 }) {
  with (b = { x: 4, get [Symbol.unscopables]() {
                      unscopablesGetterCalled++;
                      return { x: flag=!flag };
                    } }) {
    x++;
  }
}

assert.sameValue(unscopablesGetterCalled, 1);
assert.sameValue(a.x, 7);
assert.sameValue(b.x, 5);

unscopablesGetterCalled = 0;
flag = true;
with (a = { x: 7 }) {
  with (b = { x: 4, get [Symbol.unscopables]() {
                      unscopablesGetterCalled++;
                      return { x: flag=!flag };
                    } }) {
    x--;
  }
}

assert.sameValue(unscopablesGetterCalled, 1);
assert.sameValue(a.x, 7);
assert.sameValue(b.x, 3);
