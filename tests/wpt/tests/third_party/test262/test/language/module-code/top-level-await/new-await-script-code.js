// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-AwaitExpression
description: >
  await is not a keyword in script code
features: [top-level-await]
---*/

class await {}

assert.sameValue(new await instanceof await, true);
