// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import "./dep-2.2.1_FIXTURE.js"

globalThis.evaluations.push("2.2 start");

await Promise.resolve(0);

globalThis.evaluations.push("2.2 end");
