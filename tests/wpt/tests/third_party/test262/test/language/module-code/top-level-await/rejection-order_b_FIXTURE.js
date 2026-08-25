// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import "./rejection-order_b-sentinel_FIXTURE.js"; // Signal that evaluation of b's subgraph has started

import { p1 } from "./rejection-order_setup_FIXTURE.js";
await p1.promise;
