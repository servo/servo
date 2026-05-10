// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.raw
info: bugzilla.mozilla.org/show_bug.cgi?id=1039774
esid: pending
---*/

assert.throws(TypeError, function() { String.raw(); });

assert.sameValue(String.raw.length, 1);

var cooked = [];
assert.throws(TypeError, function() { String.raw(cooked); });

cooked.raw = {};
assert.sameValue(String.raw(cooked), "");

cooked.raw = {lengt: 0};
assert.sameValue(String.raw(cooked), "");

cooked.raw = {length: 0};
assert.sameValue(String.raw(cooked), "");

cooked.raw = {length: -1};
assert.sameValue(String.raw(cooked), "");

cooked.raw = [];
assert.sameValue(String.raw(cooked), "");

cooked.raw = ["a"];
assert.sameValue(String.raw(cooked), "a");

cooked.raw = ["a", "b"];
assert.sameValue(String.raw(cooked, "x"), "axb");

cooked.raw = ["a", "b"];
assert.sameValue(String.raw(cooked, "x", "y"), "axb");

cooked.raw = ["a", "b", "c"];
assert.sameValue(String.raw(cooked, "x"), "axbc");

cooked.raw = ["a", "b", "c"];
assert.sameValue(String.raw(cooked, "x", "y"), "axbyc");

cooked.raw = ["\n", "\r\n", "\r"];
assert.sameValue(String.raw(cooked, "x", "y"), "\nx\r\ny\r");

cooked.raw = ["\n", "\r\n", "\r"];
assert.sameValue(String.raw(cooked, "\r\r", "\n"), "\n\r\r\r\n\n\r");

cooked.raw = {length: 2, '0':"a", '1':"b", '2':"c"};
assert.sameValue(String.raw(cooked, "x", "y"), "axb");

cooked.raw = {length: 4, '0':"a", '1':"b", '2':"c"};
assert.sameValue(String.raw(cooked, "x", "y"), "axbycundefined");
