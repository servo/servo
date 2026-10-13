// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

export let disposed = false;

await using resource = {
  async [Symbol.asyncDispose]() {
    if (disposed) {
      throw new Error('resource disposed multiple times');
    }
    // wait a few ticks to ensure the Promise returned by this function is fully awaited before evaluation is considered complete
    await 0;
    await 0;
    await 0;
    disposed = true;
  }
};
export { resource };

if (disposed) {
  throw new Error('resource disposed before module evaluation completed');
}
