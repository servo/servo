// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements
description: >
    await using: 'for (await using of of' interpreted as 'await using'
features: [explicit-resource-management]
---*/

async function f() {
  for (await using of of []) { }
}
