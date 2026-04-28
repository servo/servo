// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    When the `yield` keyword occurs within the PropertyName of a
    non-generator MethodDefinition within a generator function, it behaves as a
    YieldExpression.
info: |
  ComputedPropertyName:
    [ AssignmentExpression ]

  AssignmentExpression[In, Yield, Await]:
    [+Yield]YieldExpression[?In, ?Await]

features: [computed-property-names, generators]
flags: [noStrict]
---*/

function * g() {
  let o = {
    [yield 10]: 1,
    a: 'a'
  };

  yield 20;
  return o;
}

let iter = g();
assert.sameValue(iter.next().value, 10);
assert.sameValue(iter.next().value, 20);

let outcome = iter.next().value;

assert.sameValue(outcome[undefined], 1);
assert.sameValue(outcome.a, 'a');

