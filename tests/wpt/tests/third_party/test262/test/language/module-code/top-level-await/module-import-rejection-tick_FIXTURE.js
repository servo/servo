// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

export default 42;

await Promise.resolve().then(() => {
  // This rejection will tick first unwrapping all the promises first
  return Promise.reject(new RangeError());
});

var rejection = Promise.reject(new TypeError());
await rejection;
