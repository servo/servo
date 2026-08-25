// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExp.prototype[@@split] should handle if lastIndex is out of bound.
info: bugzilla.mozilla.org/show_bug.cgi?id=1263851
esid: pending
---*/

var myRegExp = {
    get constructor() {
        return {
            get [Symbol.species]() {
                return function() {
                    return {
                        get lastIndex() {
                            return 9;
                        },
                        set lastIndex(v) {},
                        exec() {
                            return [];
                        }
                    };
                };
            }
        };
    }
};
var result = RegExp.prototype[Symbol.split].call(myRegExp, "abcde");;
assert.sameValue(result.length, 2);
assert.sameValue(result[0], "");
assert.sameValue(result[1], "");
