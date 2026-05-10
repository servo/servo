// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-statement
description: >
    using: 'for (using x = ' and 'for (using of =' are interpreted as for loop
features: [explicit-resource-management]
---*/

for (using x = null;;) break;

// 'using of' lookahead restriction only applies to 'for-of'/'for-await-of'. In 'for' statement it is
// handled similar to `for (let of = null;;)`:
for (using of = null;;) break;
