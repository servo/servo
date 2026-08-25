// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

export default function fn() {
  fn = 2;
  return 1;
}
