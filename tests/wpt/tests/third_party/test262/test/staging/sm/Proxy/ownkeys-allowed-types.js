// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function makeProxy(type) {
    return new Proxy({}, { ownKeys() { return [type]; } });
}

for (var type of [123, 12.5, true, false, undefined, null, {}, []]) {
    var proxy = makeProxy(type);
    assert.throws(TypeError, () => Object.ownKeys(proxy));
    assert.throws(TypeError, () => Object.getOwnPropertyNames(proxy));
}

type = Symbol();
proxy = makeProxy(type);
assert.sameValue(Object.getOwnPropertySymbols(proxy)[0], type);

type = "abc";
proxy = makeProxy(type);
assert.sameValue(Object.getOwnPropertyNames(proxy)[0], type);

