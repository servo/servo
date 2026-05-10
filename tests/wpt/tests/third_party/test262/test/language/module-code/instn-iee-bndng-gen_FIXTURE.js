// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

export { A as B } from './instn-iee-bndng-gen.js';

// Taken together, the following two assertions demonstrate that there is no
// entry in the environment record for ImportName:
export const results = [];
try {
  A;
} catch (error) {
  results.push(error.name, typeof A);
}

// Taken together, the following two assertions demonstrate that there is no
// entry in the environment record for ExportName:
try {
  B;
} catch (error) {
  results.push(error.name, typeof B);
}
