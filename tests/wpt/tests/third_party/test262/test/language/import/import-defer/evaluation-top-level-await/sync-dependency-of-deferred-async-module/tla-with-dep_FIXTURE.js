// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import "./dep_FIXTURE.js";

globalThis.evaluations.push("tla-with-dep start");

await Promise.resolve(0);

globalThis.evaluations.push("tla-with-dep end");
