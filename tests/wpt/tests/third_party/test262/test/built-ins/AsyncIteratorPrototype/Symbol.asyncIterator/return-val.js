// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asynciteratorprototype-asynciterator
description: Return value of @@asyncIterator
info: |
  %AsyncIteratorPrototype% [ @@asyncIterator ] ( )
    1. Return the this value.
features: [Symbol.asyncIterator, async-iteration]
---*/

async function* generator() {}
const AsyncIteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf(generator.prototype))
const getAsyncIterator = AsyncIteratorPrototype[Symbol.asyncIterator];

const thisValues = [
    {},
    Symbol(),
    4,
    4n,
    true,
    undefined,
    null,
];

for (const thisValue of thisValues) {
    assert.sameValue(
        getAsyncIterator.call(thisValue),
        thisValue
    );
}
