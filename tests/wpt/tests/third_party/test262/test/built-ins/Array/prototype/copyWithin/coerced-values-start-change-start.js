// Copyright (C) 2019 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  SECURITY: start argument is coerced to an integer value
  and side effects change the length of the array so that
  the start is out of bounds
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  8. Let relativeStart be ToInteger(start).
  ...
includes: [compareArray.js]
---*/


// make a long integer Array
function longDenseArray(){
    var a = [0];
    for(var i = 0; i < 1024; i++){
        a[i] = i;
    }
    return a;
}

function shorten(){
    currArray.length = 20;
    return 1000;
}

var array = [];
array.length = 20;

var currArray = longDenseArray();

assert.compareArray(
  currArray.copyWithin(0, {valueOf: shorten}), array,
  'currArray.copyWithin(0, {valueOf: shorten}) returns array'
);

currArray = longDenseArray();
Object.setPrototypeOf(currArray, longDenseArray());

var array2 = longDenseArray();
array2.length = 20;
for(var i = 0; i < 24; i++){
    array2[i] = Object.getPrototypeOf(currArray)[i+1000];
}

assert.compareArray(
  currArray.copyWithin(0, {valueOf: shorten}), array2,
  'currArray.copyWithin(0, {valueOf: shorten}) returns array2'
);
