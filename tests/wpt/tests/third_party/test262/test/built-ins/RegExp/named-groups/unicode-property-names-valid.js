// Copyright (C) 2020 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Michael Saboff
description: Exotic named group names in Unicode RegExps
esid: prod-GroupSpecifier
features: [regexp-named-groups]
---*/

/*
 Valid ID_Start / ID_Continue Unicode characters

 𝑓  \u{1d453}  \ud835 \udc53
 𝑜  \u{1d45c}  \ud835 \udc5c
 𝑥  \u{id465}  \ud835 \udc65

 𝓓  \u{1d4d3}  \ud835 \udcd3
 𝓸  \u{1d4f8}  \ud835 \udcf8
 𝓰  \u{1d4f0}  \ud835 \udcf0

 𝓑  \u{1d4d1}  \ud835 \udcd1
 𝓻  \u{1d4fb}  \ud835 \udcfb
 𝓸  \u{1d4f8}  \ud835 \udcf8
 𝔀  \u{1d500}  \ud835 \udd00
 𝓷  \u{1d4f7}  \ud835 \udcf7

 𝖰  \u{1d5b0}  \ud835 \uddb0
 𝖡  \u{1d5a1}  \ud835 \udda1
 𝖥  \u{1d5a5}  \ud835 \udda5

 (fox) 狸  \u{72f8}  \u72f8
 (dog) 狗  \u{72d7}  \u72d7  

 Valid ID_Continue Unicode characters (Can't be first identifier character.)

 𝟚  \u{1d7da}  \ud835 \udfda

 Invalid ID_Start / ID_Continue

 (fox face emoji) 🦊  \u{1f98a}  \ud83e \udd8a
 (dog emoji)  🐕  \u{1f415}  \ud83d \udc15
*/

var string = "The quick brown fox jumped over the lazy dog's back";
var string2 = "It is a dog eat dog world.";

let match = null;

assert.sameValue(string.match(/(?<animal>fox|dog)/u).groups.animal, "fox");
assert.sameValue(string.match(/(?<the2>the)/u).groups.the2, "the");

match = string.match(/(?<𝑓𝑜𝑥>fox).*(?<𝓓𝓸𝓰>dog)/u);
assert.sameValue(match.groups.𝑓𝑜𝑥, "fox");
assert.sameValue(match.groups.𝓓𝓸𝓰, "dog");
assert.sameValue(match[1], "fox");
assert.sameValue(match[2], "dog");

match = string.match(/(?<狸>fox).*(?<狗>dog)/u);
assert.sameValue(match.groups.狸, "fox");
assert.sameValue(match.groups.狗, "dog");
assert.sameValue(match[1], "fox");
assert.sameValue(match[2], "dog");

assert.sameValue(string.match(/(?<𝓑𝓻𝓸𝔀𝓷>brown)/u).groups.𝓑𝓻𝓸𝔀𝓷, "brown");
assert.sameValue(string.match(/(?<𝓑𝓻𝓸𝔀𝓷>brown)/u).groups.\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}, "brown");
assert.sameValue(string.match(/(?<\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}>brown)/u).groups.𝓑𝓻𝓸𝔀𝓷, "brown");
assert.sameValue(string.match(/(?<\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}>brown)/u).groups.\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}, "brown");
assert.sameValue(string.match(/(?<\ud835\udcd1\ud835\udcfb\ud835\udcf8\ud835\udd00\ud835\udcf7>brown)/u).groups.𝓑𝓻𝓸𝔀𝓷, "brown");
assert.sameValue(string.match(/(?<\ud835\udcd1\ud835\udcfb\ud835\udcf8\ud835\udd00\ud835\udcf7>brown)/u).groups.\u{1d4d1}\u{1d4fb}\u{1d4f8}\u{1d500}\u{1d4f7}, "brown");

assert.sameValue(string.match(/(?<𝖰𝖡𝖥>q\w*\W\w*\W\w*)/u).groups.𝖰𝖡𝖥, "quick brown fox");
assert.sameValue(string.match(/(?<𝖰𝖡\u{1d5a5}>q\w*\W\w*\W\w*)/u).groups.𝖰𝖡𝖥, "quick brown fox");
assert.sameValue(string.match(/(?<𝖰\u{1d5a1}𝖥>q\w*\W\w*\W\w*)/u).groups.𝖰𝖡𝖥, "quick brown fox");
assert.sameValue(string.match(/(?<𝖰\u{1d5a1}\u{1d5a5}>q\w*\W\w*\W\w*)/u).groups.𝖰𝖡𝖥, "quick brown fox");
assert.sameValue(string.match(/(?<\u{1d5b0}𝖡𝖥>q\w*\W\w*\W\w*)/u).groups.𝖰𝖡𝖥, "quick brown fox");
assert.sameValue(string.match(/(?<\u{1d5b0}𝖡\u{1d5a5}>q\w*\W\w*\W\w*)/u).groups.𝖰𝖡𝖥, "quick brown fox");
assert.sameValue(string.match(/(?<\u{1d5b0}\u{1d5a1}𝖥>q\w*\W\w*\W\w*)/u).groups.𝖰𝖡𝖥, "quick brown fox");
assert.sameValue(string.match(/(?<\u{1d5b0}\u{1d5a1}\u{1d5a5}>q\w*\W\w*\W\w*)/u).groups.𝖰𝖡𝖥, "quick brown fox");

assert.sameValue(string.match(/(?<the𝟚>the)/u).groups.the𝟚, "the");
assert.sameValue(string.match(/(?<the\u{1d7da}>the)/u).groups.the𝟚, "the");
assert.sameValue(string.match(/(?<the\ud835\udfda>the)/u).groups.the𝟚, "the");

match = string2.match(/(?<dog>dog)(.*?)(\k<dog>)/u);
assert.sameValue(match.groups.dog, "dog");
assert.sameValue(match[1], "dog");
assert.sameValue(match[2], " eat ");
assert.sameValue(match[3], "dog");

match = string2.match(/(?<𝓓𝓸𝓰>dog)(.*?)(\k<𝓓𝓸𝓰>)/u);
assert.sameValue(match.groups.𝓓𝓸𝓰, "dog");
assert.sameValue(match[1], "dog");
assert.sameValue(match[2], " eat ");
assert.sameValue(match[3], "dog");

match = string2.match(/(?<狗>dog)(.*?)(\k<狗>)/u);
assert.sameValue(match.groups.狗, "dog");
assert.sameValue(match[1], "dog");
assert.sameValue(match[2], " eat ");
assert.sameValue(match[3], "dog");
