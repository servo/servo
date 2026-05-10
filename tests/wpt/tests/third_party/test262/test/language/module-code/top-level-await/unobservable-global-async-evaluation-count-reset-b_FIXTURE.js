// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import { pB, pB_start } from "./unobservable-global-async-evaluation-count-reset-setup_FIXTURE.js";
pB_start.resolve();
await pB.promise;
