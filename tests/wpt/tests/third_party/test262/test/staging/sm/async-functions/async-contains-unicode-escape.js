// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  async/await containing escapes
info: bugzilla.mozilla.org/show_bug.cgi?id=1315815
esid: pending
---*/

function test(code)
{
  var unescaped = code.replace("###", "async");
  var escaped = code.replace("###", "\\u0061sync");

  assert.throws(SyntaxError, () => eval(escaped));
  eval(unescaped);
}

test("### function f() {}", eval);
test("var x = ### function f() {}", eval);
test("### x => {};", eval);
test("var x = ### x => {}", eval);
test("### () => {};", eval);
test("var x = ### () => {}", eval);
test("### (y) => {};", eval);
test("var x = ### (y) => {}", eval);
test("({ ### x() {} })", eval);
test("var x = ### function f() {}", eval);

assert.throws(SyntaxError, () => eval("async await => 1;"));
assert.throws(SyntaxError, () => eval("async aw\\u0061it => 1;"));

var async = 0;
assert.sameValue(\u0061sync, 0);

var obj = { \u0061sync() { return 1; } };
assert.sameValue(obj.async(), 1);

async = function() { return 42; };

var z = async(obj);
assert.sameValue(z, 42);

var w = async(obj)=>{};
assert.sameValue(typeof w, "function");
