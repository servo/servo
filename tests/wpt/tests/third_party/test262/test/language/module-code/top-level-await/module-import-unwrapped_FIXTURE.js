// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

await 1;

export default Promise.resolve('default');

export const x = Promise.reject('unwrapped rejection');
export const y = Promise.resolve('unwrapped resolution');
