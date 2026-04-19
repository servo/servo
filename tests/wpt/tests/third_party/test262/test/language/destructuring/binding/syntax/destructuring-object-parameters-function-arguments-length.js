// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-destructuring-binding-patterns-static-semantics-hasinitializer
description: >
  Function.length when ObjectBindingPattern in FormalParameterList
info: |
  #sec-function-definitions-static-semantics-expectedargumentcount

  Static Semantics: ExpectedArgumentCount

    FormalParameterList : FormalParameter

    1. If HasInitializer of FormalParameter is true, return 0.
    2. Return 1.

  #sec-destructuring-binding-patterns-static-semantics-hasinitializer

  Static Semantics: HasInitializer

    BindingElement : BindingPattern

    1. Return false.

features: [destructuring-binding]
---*/

assert.sameValue((({a,b}) => {}).length, 1);
assert.sameValue((function({a,b}) {}).length, 1);
assert.sameValue((function * ({a,b}) {}).length, 1);
assert.sameValue((async ({a,b}) => {}).length, 1);
assert.sameValue((async function({a,b}) {}).length, 1);
assert.sameValue((async function * ({a,b}) {}).length, 1);

