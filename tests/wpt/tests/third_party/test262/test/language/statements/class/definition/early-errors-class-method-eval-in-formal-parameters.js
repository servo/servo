// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: It is a SyntaxError if FormalParameters contains eval in strict mode
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
class Foo {
  async foo(eval) { }
}
