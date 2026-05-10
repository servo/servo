// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-keyeddestructuringassignmentevaluation
description: >
    Ensure correct evaluation order when destructuring target is property reference.
info: |
    12.15.5.2 Runtime Semantics: DestructuringAssignmentEvaluation

    AssignmentProperty : PropertyName : AssignmentElement

    1. Let name be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(name).
    3. Return the result of performing KeyedDestructuringAssignmentEvaluation of
       AssignmentElement with value and name as the arguments. 

    12.15.5.4 Runtime Semantics: KeyedDestructuringAssignmentEvaluation

    1. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an ArrayLiteral, then
        a. Let lref be the result of evaluating DestructuringAssignmentTarget.
        b. ReturnIfAbrupt(lref).
    2. Let v be ? GetV(value, propertyName).
    ...
    4. Else, let rhsValue be v.
    ...
    7. Return ? PutValue(lref, rhsValue).
includes: [compareArray.js]
---*/


var log = [];

function source() {
    log.push("source");
    return {
        get p() {
            log.push("get");
        }
    };
}
function target() {
    log.push("target");
    return {
        set q(v) {
            log.push("set");
        }
    };
}
function sourceKey() {
    log.push("source-key");
    return {
        toString: function() {
            log.push("source-key-tostring");
            return "p";
        }
    };
}
function targetKey() {
    log.push("target-key");
    return {
        toString: function() {
            log.push("target-key-tostring");
            return "q";
        }
    };
}

({[sourceKey()]: target()[targetKey()]} = source());

assert.compareArray(log, [
    "source", "source-key", "source-key-tostring",
    "target", "target-key",
    "get", "target-key-tostring", "set",
]);
