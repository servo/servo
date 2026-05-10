// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Set instances should be able to be traversed using a `for...of` loop.
es6id: 13.6.4
features: [Set]
---*/

var set = new Set();
var obj = {};
var iterationCount = 0;

var first = 0;
var second = 'a';
var third = true;
var fourth = false;
var fifth = null;
var sixth = undefined;
var seventh = NaN;
var eight = obj;

set.add(0);
set.add('a');
set.add(true);
set.add(false);
set.add(null);
set.add(undefined);
set.add(NaN);
set.add(obj);

for (var x of set) {
  assert.sameValue(x, first);
  first = second;
  second = third;
  third = fourth;
  fourth = fifth;
  fifth = sixth;
  sixth = seventh;
  seventh = eight;
  eight = null;
  iterationCount += 1;
}

assert.sameValue(iterationCount, 8);
