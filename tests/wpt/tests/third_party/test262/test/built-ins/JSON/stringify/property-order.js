// Copyright 2019 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-serializejsonobject
description: JSON.stringify property enumeration order
features: [for-in-order]
---*/

var o = {
  p1: 'p1',
  p2: 'p2',
  p3: 'p3',
};

// This getter will be triggered during enumeration, but the property it adds should not be enumerated.
Object.defineProperty(o, 'add', {
  enumerable: true,
  get: function () {
    o.extra = 'extra';
    return 'add';
  }
});

o.p4 = 'p4';

o[2] = '2';
o[0] = '0';
o[1] = '1';

delete o.p1;
delete o.p3;
o.p1 = 'p1';

var actual = JSON.stringify(o);

var expected = '{"0":"0","1":"1","2":"2","p2":"p2","add":"add","p4":"p4","p1":"p1"}';

assert.sameValue(actual, expected);
