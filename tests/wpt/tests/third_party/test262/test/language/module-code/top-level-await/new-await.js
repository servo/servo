// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-AwaitExpression
description: >
  await cannot immediately follow new in module code
negative:
  phase: parse
  type: SyntaxError
flags: [module]
features: [top-level-await]
---*/

$DONOTEVALUATE();

new await;
