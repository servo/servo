// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: >
  It is a Syntax Error if ContainsUseStrict of AsyncConciseBody is *true* and IsSimpleParameterList of ArrowParameters is *false*.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
({
  foo(x = 1) {"use strict"}
});
