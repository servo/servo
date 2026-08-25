// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Test corner cases of for-of iteration over Arrays.
// The current SetObject::construct method uses a ForOfIterator to extract
// values from the array, so we use that mechanism to test ForOfIterator here.

// Test the properties and prototype of a generator object.
function TestManySmallArrays() {
    function doIter(f, arr) {
        return f(...new Set(arr));
    }

    function fun(a, b, c) {
        var result = 0;
        for (var i = 0; i < arguments.length; i++)
            result += arguments[i];
        return result;
    }


    var TRUE_SUM = 0;
    var N = 100;
    var M = 3;
    var sum = 0;
    for (var i = 0; i < N; i++) {
        var arr = new Array(M);
        for (var j = 0; j < M; j++) {
            arr[j] = j;
            TRUE_SUM += j;
        }
        sum += doIter(fun, arr);
    }
    assert.sameValue(sum, TRUE_SUM);
}
TestManySmallArrays();

// Test the properties and prototype of a generator object.
function TestSingleSmallArray() {
    function doIter(f, arr) {
        return f(...new Set(arr));
    }

    function fun(a, b, c) {
        var result = 0;
        for (var i = 0; i < arguments.length; i++)
            result += arguments[i];
        return result;
    }


    var TRUE_SUM = 0;
    var N = 100;
    var M = 3;
    var arr = new Array(M);
    for (var j = 0; j < M; j++) {
        arr[j] = j;
        TRUE_SUM += j;
    }
    TRUE_SUM *= N;

    var sum = 0;
    for (var i = 0; i < N; i++) {
        sum += doIter(fun, arr);
    }
    assert.sameValue(sum, TRUE_SUM);
}
TestSingleSmallArray();


function TestChangeArrayPrototype() {
    function doIter(f, arr) {
        return f(...new Set(arr));
    }

    function fun(a, b, c) {
        var result = 0;
        for (var i = 0; i < arguments.length; i++)
            result += arguments[i];
        return result;
    }

    var Proto1 = Object.create(Array.prototype);

    var TRUE_SUM = 0;
    var N = 100;
    var MID = N/2;
    var M = 3;
    var arr = new Array(M);
    var ARR_SUM = 0;
    for (var j = 0; j < M; j++) {
        arr[j] = j;
        ARR_SUM += j;
    }

    var sum = 0;
    for (var i = 0; i < N; i++) {
        sum += doIter(fun, arr);
        if (i == MID)
            arr.__proto__ = Proto1;
        TRUE_SUM += ARR_SUM;
    }
    assert.sameValue(sum, TRUE_SUM);
}
TestChangeArrayPrototype();


function TestChangeManyArrayShape() {
    function doIter(f, arr) {
        return f(...new Set(arr));
    }

    function fun(a, b, c) {
        var result = 0;
        for (var i = 0; i < arguments.length; i++)
            result += arguments[i];
        return result;
    }

    var TRUE_SUM = 0;
    var N = 100;
    var MID = N/2;
    var M = 3;
    var sum = 0;
    for (var i = 0; i < N; i++) {
        var arr = new Array(M);
        var ARR_SUM = 0;
        for (var j = 0; j < M; j++) {
            arr[j] = j;
            ARR_SUM += j;
        }
        arr['v_' + i] = i;
        sum += doIter(fun, arr);
        TRUE_SUM += ARR_SUM;
    }
    assert.sameValue(sum, TRUE_SUM);
}
TestChangeManyArrayShape();

