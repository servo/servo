// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// This is still valid in script code and strict and non strict modes
// This is invalid as module code
// https://tc39.github.io/ecma262/#sec-scripts-static-semantics-lexicallydeclarednames
var smoosh; function smoosh() {}
