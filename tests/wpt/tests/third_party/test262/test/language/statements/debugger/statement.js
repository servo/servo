// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: The `debugger` token may occupy a statement position
esid: sec-debugger-statement
es6id: 13.16
---*/

// Expressing within a `while` statement ensures that the `debugger  keyword is
// not erroneously interpreted as a declaration. It also avoids statement
// evaluation, which is host-defined an may interrupt test execution.
while (false) debugger;
