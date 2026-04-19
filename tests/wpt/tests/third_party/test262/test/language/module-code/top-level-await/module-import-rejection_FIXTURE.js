// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

export const resolved = await 42;

// Can't use Test262Error in this file because it's not referenced here
export default await Promise.reject(new TypeError('error in the default export line'));

// Use RangeError to differentiate from initial error
export const x = await Promise.reject(new RangeError('named x rejection'));
export const y = await Promise.reject(new RangeError('named y rejection'));
