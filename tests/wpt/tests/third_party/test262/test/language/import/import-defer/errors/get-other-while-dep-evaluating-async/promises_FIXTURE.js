// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

export let resolveDone, rejectDone;
export const done = new Promise((r, j) => (resolveDone = r, rejectDone = j));

export let resolveFirst, rejectFirst;
export const first = new Promise((r, j) => (resolveFirst = r, rejectFirst = j));

export let resolveSecond, rejectSecond;
export const second = new Promise((r, j) => (resolveSecond = r, rejectSecond = j));

export let resolveThird, rejectThird;
export const third = new Promise((r, j) => (resolveThird = r, rejectThird = j));
