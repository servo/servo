// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import { resolveFirst, resolveThird, second } from "./promises_FIXTURE.js";
import "./dep-1.1_FIXTURE.js"

await Promise.resolve();

resolveFirst();

await second;

resolveThird();
