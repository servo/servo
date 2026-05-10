// Copyright 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-AwaitExpression
description: >
  await requries an operand.
negative:
  phase: parse
  type: SyntaxError
flags: [module]
features: [top-level-await]
---*/

$DONOTEVALUATE();

await;
