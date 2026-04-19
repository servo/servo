// Copyright 2017 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-functiondeclarationinstantiation
description: Non-strict mode function execution context has a mutable "arguments" binding, however it is created with a "false" argument, which means it may not be deleted.
info: |
  envRec.CreateMutableBinding("arguments", false).

  CreateMutableBinding(N, D)

  Create a new but uninitialized mutable binding in an Environment Record. The String value N is the text of the bound name. If the Boolean argument D is true the binding may be subsequently deleted.

flags: [noStrict]
---*/

function f1() {
  assert.sameValue(delete arguments, false);
}

f1();
