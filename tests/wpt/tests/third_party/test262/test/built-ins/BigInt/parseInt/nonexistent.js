// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: BigInt does not have a static parseInt function
features: [BigInt]
---*/

assert(!BigInt.hasOwnProperty("parseInt"));
