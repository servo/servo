// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

for (let name of ["test", Symbol.match, Symbol.replace, Symbol.search]) {
    let methodName = typeof name === "symbol" ? `[${name.description}]` : name;
    assert.throws(
        TypeError,
        () => RegExp.prototype[name].call({}),
        `${methodName} method called on incompatible Object`);
}

