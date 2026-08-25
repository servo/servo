// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

await 1;
await 2;
export default await Promise.resolve(42);

export const y = await 39;
export const x = await 'named';

// Bonus: this rejection is not unwrapped
if (false) {
  await Promise.reject(42);
}
