// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function name(obj, property, get) {
    let desc = Object.getOwnPropertyDescriptor(obj, property);
    return (get ? desc.get : desc.set).name;
}

assert.sameValue(name({get a() {}}, "a", true), "get a");
assert.sameValue(name({set a(v) {}}, "a", false), "set a");

assert.sameValue(name({get 123() {}}, "123", true), "get 123");
assert.sameValue(name({set 123(v) {}}, "123", false), "set 123");

assert.sameValue(name({get case() {}}, "case", true), "get case");
assert.sameValue(name({set case(v) {}}, "case", false), "set case");

assert.sameValue(name({get get() {}}, "get", true), "get get");
assert.sameValue(name({set set(v) {}}, "set", false), "set set");

let o = {get a() { }, set a(v) {}};
assert.sameValue(name(o, "a", true), "get a");
assert.sameValue(name(o, "a", false), "set a");

o = {get 123() { }, set 123(v) {}}
assert.sameValue(name(o, "123", true), "get 123");
assert.sameValue(name(o, "123", false), "set 123");

o = {get case() { }, set case(v) {}}
assert.sameValue(name(o, "case", true), "get case");
assert.sameValue(name(o, "case", false), "set case");

assert.sameValue(name({get ["a"]() {}}, "a", true), "get a");
assert.sameValue(name({get [123]() {}}, "123", true), "get 123");
assert.sameValue(name({set ["a"](v) {}}, "a", false), "set a");
assert.sameValue(name({set [123](v) {}}, "123", false), "set 123");

