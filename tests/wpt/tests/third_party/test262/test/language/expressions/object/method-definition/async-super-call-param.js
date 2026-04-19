// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: >
  Super calls work in parameter list of async methods
flags: [async]
---*/

var sup = {
  method() {
    return 'sup';
  }
}

var child = {
  async method(x = super.method()) {
    var y = await x;
    assert.sameValue(y, 'sup');
  }
}

Object.setPrototypeOf(child, sup);

child.method().then($DONE, $DONE);

