// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import "./dep-2.2-tla_FIXTURE.js";

globalThis.evaluations.push("4.1 start");

await Promise.resolve(0);

globalThis.evaluations.push("4.1 end");
