// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Implement RegExp.prototype.{global, ignoreCase, multiline, sticky, unicode}
info: bugzilla.mozilla.org/show_bug.cgi?id=1120169
esid: pending
---*/

var props = [
  "global",
  "ignoreCase",
  "multiline",
  "sticky",
  "unicode",
];

for (var prop of props) {
  assert.sameValue(RegExp.prototype[prop], undefined,
    `expected undefined for ${prop} on prototype`);
}

test(/foo/iymg, [true, true, true, true, false]);
test(RegExp(""), [false, false, false, false, false]);
test(RegExp("", "mygi"), [true, true, true, true, false]);
test(RegExp("", "mygiu"), [true, true, true, true, true]);

testThrowsGeneric();
testThrowsGeneric(1);
testThrowsGeneric("");
testThrowsGeneric({});
testThrowsGeneric(new Proxy({}, {get(){ return true; }}));

function test(obj, expects) {
  for (var i = 0; i < props.length; i++) {
    assert.sameValue(obj[props[i]], expects[i]);
  }
}

function testThrowsGeneric(obj) {
  for (var prop of props) {
    assert.throws(TypeError, () => genericGet(obj, prop));
  }
}

function genericGet(obj, prop) {
    return Object.getOwnPropertyDescriptor(RegExp.prototype, prop).get.call(obj);
}
