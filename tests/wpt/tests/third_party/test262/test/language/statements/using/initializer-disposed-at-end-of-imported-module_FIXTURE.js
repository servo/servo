// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

export let disposed = false;

using resource = {
  [Symbol.dispose]() {
    if (disposed) {
      throw new Error('resource disposed multiple times');
    }
    disposed = true;
  }
};
export { resource };

if (disposed) {
  throw new Error('resource disposed before module evaluation completed');
}
