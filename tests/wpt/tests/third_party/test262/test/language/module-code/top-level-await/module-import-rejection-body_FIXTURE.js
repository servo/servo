// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

export default 42;

export const named = 'named';

var rejection = Promise.reject(new TypeError('I reject this!'));
await rejection;
