// Copyright (C) 2026 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

globalThis.evaluations = [];

// Promise that keeps A suspended until resolve-blocker_FIXTURE.js runs.
export const blocker = Promise.withResolvers();

// Promise used to signal that A's evaluation has started.
export const aStarted = Promise.withResolvers();
