// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-expressions-shell.js]
description: |
  Array destructuring with various default values in various context - function expression with nested objects
info: bugzilla.mozilla.org/show_bug.cgi?id=1184922
esid: pending
---*/

testDestructuringArrayDefault("function f() { return { f() {}, *g() {}, r: /a/ }; }");
testDestructuringArrayDefault("function* g() { return { f() {}, *g() {}, r: /b/ }; }");
testDestructuringArrayDefault("() => { return { f() {}, *g() {}, r: /c/ }; }");
