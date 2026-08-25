// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var source = `class A {
  // Ensure this name parses.
  #℘;
}`;

Function(source);
