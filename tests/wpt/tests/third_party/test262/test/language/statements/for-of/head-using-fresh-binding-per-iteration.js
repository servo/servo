// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements
description: >
    ForDeclaration containing 'using' creates a fresh binding per iteration
features: [explicit-resource-management]
---*/

let f = [undefined, undefined, undefined];

const obj1 = { [Symbol.dispose]() { } };
const obj2 = { [Symbol.dispose]() { } };
const obj3 = { [Symbol.dispose]() { } };

let i = 0;
for (using x of [obj1, obj2, obj3]) {
  f[i++] = function() { return x; };
}
assert.sameValue(f[0](), obj1, "`f[0]()` returns `obj1`");
assert.sameValue(f[1](), obj2, "`f[1]()` returns `obj2`");
assert.sameValue(f[2](), obj3, "`f[2]()` returns `obj3`");
