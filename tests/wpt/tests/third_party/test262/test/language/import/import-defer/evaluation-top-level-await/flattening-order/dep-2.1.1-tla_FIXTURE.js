// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

globalThis.evaluations.push("2.1.1 start");

await Promise.resolve(0);

globalThis.evaluations.push("2.1.1 end");
