// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: >
  Async function expressions are not a simple assignment target.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

(async function foo() { } = 1)
