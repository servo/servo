// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: It is a SyntaxError if BoundNames of FormalParameters also occurs in the LexicallyDeclaredNames of AsyncFunctionBody
negative:
  phase: parse
  type: SyntaxError
features: [async-functions]
---*/

$DONOTEVALUATE();

({
  async function foo(bar) { let bar; }
})
