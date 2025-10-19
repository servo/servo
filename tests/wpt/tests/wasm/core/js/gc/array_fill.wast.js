(function array_fill_wast_js() {

// array_fill.wast:5
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8a\x80\x80\x80\x00\x02\x5e\x78\x00\x60\x02\x64\x00\x7f\x00\x03\x82\x80\x80\x80\x00\x01\x01\x07\x98\x80\x80\x80\x00\x01\x14\x61\x72\x72\x61\x79\x2e\x66\x69\x6c\x6c\x2d\x69\x6d\x6d\x75\x74\x61\x62\x6c\x65\x00\x00\x0a\x93\x80\x80\x80\x00\x01\x8d\x80\x80\x80\x00\x00\x20\x00\x41\x00\x20\x01\x41\x00\xfb\x10\x00\x0b", "array_fill.wast:5");

// array_fill.wast:16
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8a\x80\x80\x80\x00\x02\x5e\x78\x01\x60\x02\x64\x00\x70\x00\x03\x82\x80\x80\x80\x00\x01\x01\x07\x98\x80\x80\x80\x00\x01\x14\x61\x72\x72\x61\x79\x2e\x66\x69\x6c\x6c\x2d\x69\x6e\x76\x61\x6c\x69\x64\x2d\x31\x00\x00\x0a\x93\x80\x80\x80\x00\x01\x8d\x80\x80\x80\x00\x00\x20\x00\x41\x00\x20\x01\x41\x00\xfb\x10\x00\x0b", "array_fill.wast:16");

// array_fill.wast:27
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8a\x80\x80\x80\x00\x02\x5e\x70\x01\x60\x02\x64\x00\x7f\x00\x03\x82\x80\x80\x80\x00\x01\x01\x07\x98\x80\x80\x80\x00\x01\x14\x61\x72\x72\x61\x79\x2e\x66\x69\x6c\x6c\x2d\x69\x6e\x76\x61\x6c\x69\x64\x2d\x31\x00\x00\x0a\x93\x80\x80\x80\x00\x01\x8d\x80\x80\x80\x00\x00\x20\x00\x41\x00\x20\x01\x41\x00\xfb\x10\x00\x0b", "array_fill.wast:27");

// array_fill.wast:38
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x95\x80\x80\x80\x00\x05\x5e\x78\x00\x5e\x78\x01\x60\x01\x7f\x01\x7f\x60\x00\x00\x60\x03\x7f\x7f\x7f\x00\x03\x84\x80\x80\x80\x00\x03\x02\x03\x04\x06\x95\x80\x80\x80\x00\x02\x64\x00\x00\x41\x0a\x41\x0c\xfb\x06\x00\x0b\x64\x01\x01\x41\x0c\xfb\x07\x01\x0b\x07\xb0\x80\x80\x80\x00\x03\x0d\x61\x72\x72\x61\x79\x5f\x67\x65\x74\x5f\x6e\x74\x68\x00\x00\x0f\x61\x72\x72\x61\x79\x5f\x66\x69\x6c\x6c\x2d\x6e\x75\x6c\x6c\x00\x01\x0a\x61\x72\x72\x61\x79\x5f\x66\x69\x6c\x6c\x00\x02\x0a\xb3\x80\x80\x80\x00\x03\x89\x80\x80\x80\x00\x00\x23\x01\x20\x00\xfb\x0d\x01\x0b\x8d\x80\x80\x80\x00\x00\xd0\x01\x41\x00\x41\x00\x41\x00\xfb\x10\x01\x0b\x8d\x80\x80\x80\x00\x00\x23\x01\x20\x00\x20\x01\x20\x02\xfb\x10\x01\x0b", "array_fill.wast:38");

// array_fill.wast:38
let $1 = instance($$1);

// array_fill.wast:59
assert_trap(() => call($1, "array_fill-null", []), "array_fill.wast:59");

// array_fill.wast:62
assert_trap(() => call($1, "array_fill", [13, 0, 0]), "array_fill.wast:62");

// array_fill.wast:65
assert_trap(() => call($1, "array_fill", [0, 0, 13]), "array_fill.wast:65");

// array_fill.wast:68
assert_return(() => call($1, "array_fill", [12, 0, 0]), "array_fill.wast:68");

// array_fill.wast:71
assert_return(() => call($1, "array_get_nth", [0]), "array_fill.wast:71", 0);

// array_fill.wast:72
assert_return(() => call($1, "array_get_nth", [5]), "array_fill.wast:72", 0);

// array_fill.wast:73
assert_return(() => call($1, "array_get_nth", [11]), "array_fill.wast:73", 0);

// array_fill.wast:74
assert_trap(() => call($1, "array_get_nth", [12]), "array_fill.wast:74");

// array_fill.wast:77
assert_return(() => call($1, "array_fill", [2, 11, 2]), "array_fill.wast:77");

// array_fill.wast:78
assert_return(() => call($1, "array_get_nth", [1]), "array_fill.wast:78", 0);

// array_fill.wast:79
assert_return(() => call($1, "array_get_nth", [2]), "array_fill.wast:79", 11);

// array_fill.wast:80
assert_return(() => call($1, "array_get_nth", [3]), "array_fill.wast:80", 11);

// array_fill.wast:81
assert_return(() => call($1, "array_get_nth", [4]), "array_fill.wast:81", 0);

// array_fill.wast:84
assert_return(() => call($1, "array_fill", [0, 42, 12]), "array_fill.wast:84");

// array_fill.wast:85
assert_return(() => call($1, "array_get_nth", [0]), "array_fill.wast:85", 42);

// array_fill.wast:86
assert_return(() => call($1, "array_get_nth", [2]), "array_fill.wast:86", 42);

// array_fill.wast:87
assert_return(() => call($1, "array_get_nth", [5]), "array_fill.wast:87", 42);

// array_fill.wast:88
assert_return(() => call($1, "array_get_nth", [11]), "array_fill.wast:88", 42);

// array_fill.wast:91
assert_return(() => call($1, "array_fill", [0, 7, 1]), "array_fill.wast:91");

// array_fill.wast:92
assert_return(() => call($1, "array_get_nth", [0]), "array_fill.wast:92", 7);

// array_fill.wast:93
assert_return(() => call($1, "array_get_nth", [1]), "array_fill.wast:93", 42);

// array_fill.wast:94
assert_return(() => call($1, "array_get_nth", [11]), "array_fill.wast:94", 42);

// array_fill.wast:97
assert_return(() => call($1, "array_fill", [10, 9, 2]), "array_fill.wast:97");

// array_fill.wast:98
assert_return(() => call($1, "array_get_nth", [9]), "array_fill.wast:98", 42);

// array_fill.wast:99
assert_return(() => call($1, "array_get_nth", [10]), "array_fill.wast:99", 9);

// array_fill.wast:100
assert_return(() => call($1, "array_get_nth", [11]), "array_fill.wast:100", 9);
reinitializeRegistry();
})();
