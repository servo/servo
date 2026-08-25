// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
const otherGlobal = $262.createRealm().global;

let regExp = otherGlobal.eval("/a(b|c)/iy");

function get(name) {
    const descriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, name);
    return descriptor.get.call(regExp);
}

assert.sameValue(get("flags"), "iy");
assert.sameValue(get("global"), false);
assert.sameValue(get("ignoreCase"), true);
assert.sameValue(get("multiline"), false);
assert.sameValue(get("dotAll"), false);
assert.sameValue(get("source"), "a(b|c)");
assert.sameValue(get("sticky"), true);
assert.sameValue(get("unicode"), false);

regExp = otherGlobal.eval("new RegExp('', 'gu')");

assert.sameValue(get("flags"), "gu");
assert.sameValue(get("global"), true);
assert.sameValue(get("ignoreCase"), false);
assert.sameValue(get("multiline"), false);
assert.sameValue(get("dotAll"), false);
assert.sameValue(get("source"), "(?:)");
assert.sameValue(get("sticky"), false);
assert.sameValue(get("unicode"), true);

// Trigger escaping
regExp = otherGlobal.eval("new RegExp('a/b', '')");

assert.sameValue(get("flags"), "");
assert.sameValue(get("global"), false);
assert.sameValue(get("ignoreCase"), false);
assert.sameValue(get("multiline"), false);
assert.sameValue(get("dotAll"), false);
assert.sameValue(get("source"), "a\\/b");
assert.sameValue(get("sticky"), false);
assert.sameValue(get("unicode"), false);

