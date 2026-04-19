// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
const t = RegExp.prototype;

let properties = "toString,compile,exec,test," +
                 "flags,dotAll,global,hasIndices,ignoreCase,multiline,source,sticky,unicode,unicodeSets," +
                 "constructor," +
                 "Symbol(Symbol.match),Symbol(Symbol.replace),Symbol(Symbol.search),Symbol(Symbol.split)";
if (Object.prototype.toSource) {
    properties = "toSource," + properties;
}
if (Symbol.matchAll) {
    properties += ",Symbol(Symbol.matchAll)";
}
assert.compareArray(Reflect.ownKeys(t).map(String).sort(), properties.split(",").sort());


// Invoking getters on the prototype should not throw
function getter(name) {
    return Object.getOwnPropertyDescriptor(t, name).get.call(t);
}

assert.sameValue(getter("flags"), "");
assert.sameValue(getter("global"), undefined);
assert.sameValue(getter("ignoreCase"), undefined);
assert.sameValue(getter("multiline"), undefined);
assert.sameValue(getter("source"), "(?:)");
assert.sameValue(getter("sticky"), undefined);
assert.sameValue(getter("unicode"), undefined);

assert.sameValue(t.toString(), "/(?:)/");

// The methods don't work with the prototype
assert.throws(TypeError, () => t.compile("b", "i"));
assert.throws(TypeError, () => t.test("x"));
assert.throws(TypeError, () => t.exec("x"));

