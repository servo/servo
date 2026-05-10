// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Side-effects when calling ToLength(regExp.lastIndex) in
// RegExp.prototype[@@match] for non-global RegExp can recompile the RegExp.

for (var flag of ["", "y"]) {
    var regExp = new RegExp("a", flag);

    regExp.lastIndex = {
        valueOf() {
            regExp.compile("b");
            return 0;
        }
    };

    var result = regExp[Symbol.match]("b");
    assert.sameValue(result !== null, true);
}

// Recompilation modifies flag:
// Case 1: Adds global flag, validate by checking lastIndex.
var regExp = new RegExp("a", "");
regExp.lastIndex = {
    valueOf() {
        // |regExp| is now in global mode, RegExpBuiltinExec should update the
        // lastIndex property to reflect last match.
        regExp.compile("a", "g");
        return 0;
    }
};
regExp[Symbol.match]("a");
assert.sameValue(regExp.lastIndex, 1);

// Case 2: Removes sticky flag with match, validate by checking lastIndex.
var regExp = new RegExp("a", "y");
regExp.lastIndex = {
    valueOf() {
        // |regExp| is no longer sticky, RegExpBuiltinExec shouldn't modify the
        // lastIndex property.
        regExp.compile("a", "");
        regExp.lastIndex = 9000;
        return 0;
    }
};
regExp[Symbol.match]("a");
assert.sameValue(regExp.lastIndex, 9000);

// Case 3.a: Removes sticky flag without match, validate by checking lastIndex.
var regExp = new RegExp("a", "y");
regExp.lastIndex = {
    valueOf() {
        // |regExp| is no longer sticky, RegExpBuiltinExec shouldn't modify the
        // lastIndex property.
        regExp.compile("b", "");
        regExp.lastIndex = 9001;
        return 0;
    }
};
regExp[Symbol.match]("a");
assert.sameValue(regExp.lastIndex, 9001);

// Case 3.b: Removes sticky flag without match, validate by checking lastIndex.
var regExp = new RegExp("a", "y");
regExp.lastIndex = {
    valueOf() {
        // |regExp| is no longer sticky, RegExpBuiltinExec shouldn't modify the
        // lastIndex property.
        regExp.compile("b", "");
        regExp.lastIndex = 9002;
        return 10000;
    }
};
regExp[Symbol.match]("a");
assert.sameValue(regExp.lastIndex, 9002);

