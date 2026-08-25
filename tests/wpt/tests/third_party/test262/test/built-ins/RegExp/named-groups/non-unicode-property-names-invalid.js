// Copyright (C) 2020 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Michael Saboff
description: Invalid exotic named group names in non-Unicode RegExps
esid: prod-GroupSpecifier
features: [regexp-named-groups]
---*/

/*
 Valid ID_Continue Unicode characters (Can't be first identifier character.)

 𝟚  \u{1d7da}  \ud835 \udfda

 Invalid ID_Start / ID_Continue

 (fox face emoji) 🦊  \u{1f98a}  \ud83e \udd8a
 (dog emoji)  🐕  \u{1f415}  \ud83d \udc15
*/

assert.throws(SyntaxError, function() {
    return new RegExp("(?<🦊>fox)");
});

assert.throws(SyntaxError, function() {
    return new RegExp("(?<\u{1f98a}>fox)");
});

assert.throws(SyntaxError, function() {
    return new RegExp("(?<\ud83e\udd8a>fox)");
});

assert.throws(SyntaxError, function() {
    return new RegExp("(?<🐕>dog)");
});

assert.throws(SyntaxError, function() {
    return new RegExp("(?<\u{1f415}>dog)");
});

assert.throws(SyntaxError, function() {
    return new RegExp("(?<\ud83d \udc15>dog)");
});

assert.throws(SyntaxError, function() {
    return new RegExp("(?<𝟚the>the)");
});

assert.throws(SyntaxError, function() {
    return new RegExp("(?<\u{1d7da}the>the)");
});

assert.throws(SyntaxError, function() {
    return new RegExp("(?<\ud835\udfdathe>the)");
});
