// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-expressions-shell.js]
description: |
  Array destructuring with various default values in various context - call/new expression
info: bugzilla.mozilla.org/show_bug.cgi?id=1184922
esid: pending
---*/

testDestructuringArrayDefault("func()");
testDestructuringArrayDefault("new func()");
