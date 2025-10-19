(function i64_wast_js() {

// i64.wast:3
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x97\x80\x80\x80\x00\x04\x60\x02\x7e\x7e\x01\x7e\x60\x01\x7e\x01\x7e\x60\x01\x7e\x01\x7f\x60\x02\x7e\x7e\x01\x7f\x03\xa1\x80\x80\x80\x00\x20\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x01\x01\x01\x01\x01\x02\x03\x03\x03\x03\x03\x03\x03\x03\x03\x03\x07\xeb\x81\x80\x80\x00\x20\x03\x61\x64\x64\x00\x00\x03\x73\x75\x62\x00\x01\x03\x6d\x75\x6c\x00\x02\x05\x64\x69\x76\x5f\x73\x00\x03\x05\x64\x69\x76\x5f\x75\x00\x04\x05\x72\x65\x6d\x5f\x73\x00\x05\x05\x72\x65\x6d\x5f\x75\x00\x06\x03\x61\x6e\x64\x00\x07\x02\x6f\x72\x00\x08\x03\x78\x6f\x72\x00\x09\x03\x73\x68\x6c\x00\x0a\x05\x73\x68\x72\x5f\x73\x00\x0b\x05\x73\x68\x72\x5f\x75\x00\x0c\x04\x72\x6f\x74\x6c\x00\x0d\x04\x72\x6f\x74\x72\x00\x0e\x03\x63\x6c\x7a\x00\x0f\x03\x63\x74\x7a\x00\x10\x06\x70\x6f\x70\x63\x6e\x74\x00\x11\x09\x65\x78\x74\x65\x6e\x64\x38\x5f\x73\x00\x12\x0a\x65\x78\x74\x65\x6e\x64\x31\x36\x5f\x73\x00\x13\x0a\x65\x78\x74\x65\x6e\x64\x33\x32\x5f\x73\x00\x14\x03\x65\x71\x7a\x00\x15\x02\x65\x71\x00\x16\x02\x6e\x65\x00\x17\x04\x6c\x74\x5f\x73\x00\x18\x04\x6c\x74\x5f\x75\x00\x19\x04\x6c\x65\x5f\x73\x00\x1a\x04\x6c\x65\x5f\x75\x00\x1b\x04\x67\x74\x5f\x73\x00\x1c\x04\x67\x74\x5f\x75\x00\x1d\x04\x67\x65\x5f\x73\x00\x1e\x04\x67\x65\x5f\x75\x00\x1f\x0a\xf3\x82\x80\x80\x00\x20\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x7c\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x7d\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x7e\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x7f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x80\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x81\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x82\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x83\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x84\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x85\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x86\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x87\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x88\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x89\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x8a\x0b\x85\x80\x80\x80\x00\x00\x20\x00\x79\x0b\x85\x80\x80\x80\x00\x00\x20\x00\x7a\x0b\x85\x80\x80\x80\x00\x00\x20\x00\x7b\x0b\x85\x80\x80\x80\x00\x00\x20\x00\xc2\x0b\x85\x80\x80\x80\x00\x00\x20\x00\xc3\x0b\x85\x80\x80\x80\x00\x00\x20\x00\xc4\x0b\x85\x80\x80\x80\x00\x00\x20\x00\x50\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x51\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x52\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x53\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x54\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x57\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x58\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x55\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x56\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x59\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x5a\x0b", "i64.wast:3");

// i64.wast:3
let $1 = instance($$1);

// i64.wast:38
assert_return(() => call($1, "add", [1n, 1n]), "i64.wast:38", 2n);

// i64.wast:39
assert_return(() => call($1, "add", [1n, 0n]), "i64.wast:39", 1n);

// i64.wast:40
assert_return(() => call($1, "add", [-1n, -1n]), "i64.wast:40", -2n);

// i64.wast:41
assert_return(() => call($1, "add", [-1n, 1n]), "i64.wast:41", 0n);

// i64.wast:42
assert_return(() => call($1, "add", [9_223_372_036_854_775_807n, 1n]), "i64.wast:42", -9_223_372_036_854_775_808n);

// i64.wast:43
assert_return(() => call($1, "add", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:43", 9_223_372_036_854_775_807n);

// i64.wast:44
assert_return(() => call($1, "add", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:44", 0n);

// i64.wast:45
assert_return(() => call($1, "add", [1_073_741_823n, 1n]), "i64.wast:45", 1_073_741_824n);

// i64.wast:47
assert_return(() => call($1, "sub", [1n, 1n]), "i64.wast:47", 0n);

// i64.wast:48
assert_return(() => call($1, "sub", [1n, 0n]), "i64.wast:48", 1n);

// i64.wast:49
assert_return(() => call($1, "sub", [-1n, -1n]), "i64.wast:49", 0n);

// i64.wast:50
assert_return(() => call($1, "sub", [9_223_372_036_854_775_807n, -1n]), "i64.wast:50", -9_223_372_036_854_775_808n);

// i64.wast:51
assert_return(() => call($1, "sub", [-9_223_372_036_854_775_808n, 1n]), "i64.wast:51", 9_223_372_036_854_775_807n);

// i64.wast:52
assert_return(() => call($1, "sub", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:52", 0n);

// i64.wast:53
assert_return(() => call($1, "sub", [1_073_741_823n, -1n]), "i64.wast:53", 1_073_741_824n);

// i64.wast:55
assert_return(() => call($1, "mul", [1n, 1n]), "i64.wast:55", 1n);

// i64.wast:56
assert_return(() => call($1, "mul", [1n, 0n]), "i64.wast:56", 0n);

// i64.wast:57
assert_return(() => call($1, "mul", [-1n, -1n]), "i64.wast:57", 1n);

// i64.wast:58
assert_return(() => call($1, "mul", [1_152_921_504_606_846_976n, 4_096n]), "i64.wast:58", 0n);

// i64.wast:59
assert_return(() => call($1, "mul", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:59", 0n);

// i64.wast:60
assert_return(() => call($1, "mul", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:60", -9_223_372_036_854_775_808n);

// i64.wast:61
assert_return(() => call($1, "mul", [9_223_372_036_854_775_807n, -1n]), "i64.wast:61", -9_223_372_036_854_775_807n);

// i64.wast:62
assert_return(() => call($1, "mul", [81_985_529_216_486_895n, -81_985_529_216_486_896n]), "i64.wast:62", 2_465_395_958_572_223_728n);

// i64.wast:63
assert_return(() => call($1, "mul", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:63", 1n);

// i64.wast:65
assert_trap(() => call($1, "div_s", [1n, 0n]), "i64.wast:65");

// i64.wast:66
assert_trap(() => call($1, "div_s", [0n, 0n]), "i64.wast:66");

// i64.wast:67
assert_trap(() => call($1, "div_s", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:67");

// i64.wast:68
assert_trap(() => call($1, "div_s", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:68");

// i64.wast:69
assert_return(() => call($1, "div_s", [1n, 1n]), "i64.wast:69", 1n);

// i64.wast:70
assert_return(() => call($1, "div_s", [0n, 1n]), "i64.wast:70", 0n);

// i64.wast:71
assert_return(() => call($1, "div_s", [0n, -1n]), "i64.wast:71", 0n);

// i64.wast:72
assert_return(() => call($1, "div_s", [-1n, -1n]), "i64.wast:72", 1n);

// i64.wast:73
assert_return(() => call($1, "div_s", [-9_223_372_036_854_775_808n, 2n]), "i64.wast:73", -4_611_686_018_427_387_904n);

// i64.wast:74
assert_return(() => call($1, "div_s", [-9_223_372_036_854_775_807n, 1_000n]), "i64.wast:74", -9_223_372_036_854_775n);

// i64.wast:75
assert_return(() => call($1, "div_s", [5n, 2n]), "i64.wast:75", 2n);

// i64.wast:76
assert_return(() => call($1, "div_s", [-5n, 2n]), "i64.wast:76", -2n);

// i64.wast:77
assert_return(() => call($1, "div_s", [5n, -2n]), "i64.wast:77", -2n);

// i64.wast:78
assert_return(() => call($1, "div_s", [-5n, -2n]), "i64.wast:78", 2n);

// i64.wast:79
assert_return(() => call($1, "div_s", [7n, 3n]), "i64.wast:79", 2n);

// i64.wast:80
assert_return(() => call($1, "div_s", [-7n, 3n]), "i64.wast:80", -2n);

// i64.wast:81
assert_return(() => call($1, "div_s", [7n, -3n]), "i64.wast:81", -2n);

// i64.wast:82
assert_return(() => call($1, "div_s", [-7n, -3n]), "i64.wast:82", 2n);

// i64.wast:83
assert_return(() => call($1, "div_s", [11n, 5n]), "i64.wast:83", 2n);

// i64.wast:84
assert_return(() => call($1, "div_s", [17n, 7n]), "i64.wast:84", 2n);

// i64.wast:86
assert_trap(() => call($1, "div_u", [1n, 0n]), "i64.wast:86");

// i64.wast:87
assert_trap(() => call($1, "div_u", [0n, 0n]), "i64.wast:87");

// i64.wast:88
assert_return(() => call($1, "div_u", [1n, 1n]), "i64.wast:88", 1n);

// i64.wast:89
assert_return(() => call($1, "div_u", [0n, 1n]), "i64.wast:89", 0n);

// i64.wast:90
assert_return(() => call($1, "div_u", [-1n, -1n]), "i64.wast:90", 1n);

// i64.wast:91
assert_return(() => call($1, "div_u", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:91", 0n);

// i64.wast:92
assert_return(() => call($1, "div_u", [-9_223_372_036_854_775_808n, 2n]), "i64.wast:92", 4_611_686_018_427_387_904n);

// i64.wast:93
assert_return(() => call($1, "div_u", [-8_074_936_608_141_340_688n, 4_294_967_297n]), "i64.wast:93", 2_414_874_607n);

// i64.wast:94
assert_return(() => call($1, "div_u", [-9_223_372_036_854_775_807n, 1_000n]), "i64.wast:94", 9_223_372_036_854_775n);

// i64.wast:95
assert_return(() => call($1, "div_u", [5n, 2n]), "i64.wast:95", 2n);

// i64.wast:96
assert_return(() => call($1, "div_u", [-5n, 2n]), "i64.wast:96", 9_223_372_036_854_775_805n);

// i64.wast:97
assert_return(() => call($1, "div_u", [5n, -2n]), "i64.wast:97", 0n);

// i64.wast:98
assert_return(() => call($1, "div_u", [-5n, -2n]), "i64.wast:98", 0n);

// i64.wast:99
assert_return(() => call($1, "div_u", [7n, 3n]), "i64.wast:99", 2n);

// i64.wast:100
assert_return(() => call($1, "div_u", [11n, 5n]), "i64.wast:100", 2n);

// i64.wast:101
assert_return(() => call($1, "div_u", [17n, 7n]), "i64.wast:101", 2n);

// i64.wast:103
assert_trap(() => call($1, "rem_s", [1n, 0n]), "i64.wast:103");

// i64.wast:104
assert_trap(() => call($1, "rem_s", [0n, 0n]), "i64.wast:104");

// i64.wast:105
assert_return(() => call($1, "rem_s", [9_223_372_036_854_775_807n, -1n]), "i64.wast:105", 0n);

// i64.wast:106
assert_return(() => call($1, "rem_s", [1n, 1n]), "i64.wast:106", 0n);

// i64.wast:107
assert_return(() => call($1, "rem_s", [0n, 1n]), "i64.wast:107", 0n);

// i64.wast:108
assert_return(() => call($1, "rem_s", [0n, -1n]), "i64.wast:108", 0n);

// i64.wast:109
assert_return(() => call($1, "rem_s", [-1n, -1n]), "i64.wast:109", 0n);

// i64.wast:110
assert_return(() => call($1, "rem_s", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:110", 0n);

// i64.wast:111
assert_return(() => call($1, "rem_s", [-9_223_372_036_854_775_808n, 2n]), "i64.wast:111", 0n);

// i64.wast:112
assert_return(() => call($1, "rem_s", [-9_223_372_036_854_775_807n, 1_000n]), "i64.wast:112", -807n);

// i64.wast:113
assert_return(() => call($1, "rem_s", [5n, 2n]), "i64.wast:113", 1n);

// i64.wast:114
assert_return(() => call($1, "rem_s", [-5n, 2n]), "i64.wast:114", -1n);

// i64.wast:115
assert_return(() => call($1, "rem_s", [5n, -2n]), "i64.wast:115", 1n);

// i64.wast:116
assert_return(() => call($1, "rem_s", [-5n, -2n]), "i64.wast:116", -1n);

// i64.wast:117
assert_return(() => call($1, "rem_s", [7n, 3n]), "i64.wast:117", 1n);

// i64.wast:118
assert_return(() => call($1, "rem_s", [-7n, 3n]), "i64.wast:118", -1n);

// i64.wast:119
assert_return(() => call($1, "rem_s", [7n, -3n]), "i64.wast:119", 1n);

// i64.wast:120
assert_return(() => call($1, "rem_s", [-7n, -3n]), "i64.wast:120", -1n);

// i64.wast:121
assert_return(() => call($1, "rem_s", [11n, 5n]), "i64.wast:121", 1n);

// i64.wast:122
assert_return(() => call($1, "rem_s", [17n, 7n]), "i64.wast:122", 3n);

// i64.wast:124
assert_trap(() => call($1, "rem_u", [1n, 0n]), "i64.wast:124");

// i64.wast:125
assert_trap(() => call($1, "rem_u", [0n, 0n]), "i64.wast:125");

// i64.wast:126
assert_return(() => call($1, "rem_u", [1n, 1n]), "i64.wast:126", 0n);

// i64.wast:127
assert_return(() => call($1, "rem_u", [0n, 1n]), "i64.wast:127", 0n);

// i64.wast:128
assert_return(() => call($1, "rem_u", [-1n, -1n]), "i64.wast:128", 0n);

// i64.wast:129
assert_return(() => call($1, "rem_u", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:129", -9_223_372_036_854_775_808n);

// i64.wast:130
assert_return(() => call($1, "rem_u", [-9_223_372_036_854_775_808n, 2n]), "i64.wast:130", 0n);

// i64.wast:131
assert_return(() => call($1, "rem_u", [-8_074_936_608_141_340_688n, 4_294_967_297n]), "i64.wast:131", 2_147_483_649n);

// i64.wast:132
assert_return(() => call($1, "rem_u", [-9_223_372_036_854_775_807n, 1_000n]), "i64.wast:132", 809n);

// i64.wast:133
assert_return(() => call($1, "rem_u", [5n, 2n]), "i64.wast:133", 1n);

// i64.wast:134
assert_return(() => call($1, "rem_u", [-5n, 2n]), "i64.wast:134", 1n);

// i64.wast:135
assert_return(() => call($1, "rem_u", [5n, -2n]), "i64.wast:135", 5n);

// i64.wast:136
assert_return(() => call($1, "rem_u", [-5n, -2n]), "i64.wast:136", -5n);

// i64.wast:137
assert_return(() => call($1, "rem_u", [7n, 3n]), "i64.wast:137", 1n);

// i64.wast:138
assert_return(() => call($1, "rem_u", [11n, 5n]), "i64.wast:138", 1n);

// i64.wast:139
assert_return(() => call($1, "rem_u", [17n, 7n]), "i64.wast:139", 3n);

// i64.wast:141
assert_return(() => call($1, "and", [1n, 0n]), "i64.wast:141", 0n);

// i64.wast:142
assert_return(() => call($1, "and", [0n, 1n]), "i64.wast:142", 0n);

// i64.wast:143
assert_return(() => call($1, "and", [1n, 1n]), "i64.wast:143", 1n);

// i64.wast:144
assert_return(() => call($1, "and", [0n, 0n]), "i64.wast:144", 0n);

// i64.wast:145
assert_return(() => call($1, "and", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:145", 0n);

// i64.wast:146
assert_return(() => call($1, "and", [9_223_372_036_854_775_807n, -1n]), "i64.wast:146", 9_223_372_036_854_775_807n);

// i64.wast:147
assert_return(() => call($1, "and", [4_042_326_015n, 4_294_963_440n]), "i64.wast:147", 4_042_322_160n);

// i64.wast:148
assert_return(() => call($1, "and", [-1n, -1n]), "i64.wast:148", -1n);

// i64.wast:150
assert_return(() => call($1, "or", [1n, 0n]), "i64.wast:150", 1n);

// i64.wast:151
assert_return(() => call($1, "or", [0n, 1n]), "i64.wast:151", 1n);

// i64.wast:152
assert_return(() => call($1, "or", [1n, 1n]), "i64.wast:152", 1n);

// i64.wast:153
assert_return(() => call($1, "or", [0n, 0n]), "i64.wast:153", 0n);

// i64.wast:154
assert_return(() => call($1, "or", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:154", -1n);

// i64.wast:155
assert_return(() => call($1, "or", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:155", -9_223_372_036_854_775_808n);

// i64.wast:156
assert_return(() => call($1, "or", [4_042_326_015n, 4_294_963_440n]), "i64.wast:156", 4_294_967_295n);

// i64.wast:157
assert_return(() => call($1, "or", [-1n, -1n]), "i64.wast:157", -1n);

// i64.wast:159
assert_return(() => call($1, "xor", [1n, 0n]), "i64.wast:159", 1n);

// i64.wast:160
assert_return(() => call($1, "xor", [0n, 1n]), "i64.wast:160", 1n);

// i64.wast:161
assert_return(() => call($1, "xor", [1n, 1n]), "i64.wast:161", 0n);

// i64.wast:162
assert_return(() => call($1, "xor", [0n, 0n]), "i64.wast:162", 0n);

// i64.wast:163
assert_return(() => call($1, "xor", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:163", -1n);

// i64.wast:164
assert_return(() => call($1, "xor", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:164", -9_223_372_036_854_775_808n);

// i64.wast:165
assert_return(() => call($1, "xor", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:165", 9_223_372_036_854_775_807n);

// i64.wast:166
assert_return(() => call($1, "xor", [-1n, 9_223_372_036_854_775_807n]), "i64.wast:166", -9_223_372_036_854_775_808n);

// i64.wast:167
assert_return(() => call($1, "xor", [4_042_326_015n, 4_294_963_440n]), "i64.wast:167", 252_645_135n);

// i64.wast:168
assert_return(() => call($1, "xor", [-1n, -1n]), "i64.wast:168", 0n);

// i64.wast:170
assert_return(() => call($1, "shl", [1n, 1n]), "i64.wast:170", 2n);

// i64.wast:171
assert_return(() => call($1, "shl", [1n, 0n]), "i64.wast:171", 1n);

// i64.wast:172
assert_return(() => call($1, "shl", [9_223_372_036_854_775_807n, 1n]), "i64.wast:172", -2n);

// i64.wast:173
assert_return(() => call($1, "shl", [-1n, 1n]), "i64.wast:173", -2n);

// i64.wast:174
assert_return(() => call($1, "shl", [-9_223_372_036_854_775_808n, 1n]), "i64.wast:174", 0n);

// i64.wast:175
assert_return(() => call($1, "shl", [4_611_686_018_427_387_904n, 1n]), "i64.wast:175", -9_223_372_036_854_775_808n);

// i64.wast:176
assert_return(() => call($1, "shl", [1n, 63n]), "i64.wast:176", -9_223_372_036_854_775_808n);

// i64.wast:177
assert_return(() => call($1, "shl", [1n, 64n]), "i64.wast:177", 1n);

// i64.wast:178
assert_return(() => call($1, "shl", [1n, 65n]), "i64.wast:178", 2n);

// i64.wast:179
assert_return(() => call($1, "shl", [1n, -1n]), "i64.wast:179", -9_223_372_036_854_775_808n);

// i64.wast:180
assert_return(() => call($1, "shl", [1n, 9_223_372_036_854_775_807n]), "i64.wast:180", -9_223_372_036_854_775_808n);

// i64.wast:182
assert_return(() => call($1, "shr_s", [1n, 1n]), "i64.wast:182", 0n);

// i64.wast:183
assert_return(() => call($1, "shr_s", [1n, 0n]), "i64.wast:183", 1n);

// i64.wast:184
assert_return(() => call($1, "shr_s", [-1n, 1n]), "i64.wast:184", -1n);

// i64.wast:185
assert_return(() => call($1, "shr_s", [9_223_372_036_854_775_807n, 1n]), "i64.wast:185", 4_611_686_018_427_387_903n);

// i64.wast:186
assert_return(() => call($1, "shr_s", [-9_223_372_036_854_775_808n, 1n]), "i64.wast:186", -4_611_686_018_427_387_904n);

// i64.wast:187
assert_return(() => call($1, "shr_s", [4_611_686_018_427_387_904n, 1n]), "i64.wast:187", 2_305_843_009_213_693_952n);

// i64.wast:188
assert_return(() => call($1, "shr_s", [1n, 64n]), "i64.wast:188", 1n);

// i64.wast:189
assert_return(() => call($1, "shr_s", [1n, 65n]), "i64.wast:189", 0n);

// i64.wast:190
assert_return(() => call($1, "shr_s", [1n, -1n]), "i64.wast:190", 0n);

// i64.wast:191
assert_return(() => call($1, "shr_s", [1n, 9_223_372_036_854_775_807n]), "i64.wast:191", 0n);

// i64.wast:192
assert_return(() => call($1, "shr_s", [1n, -9_223_372_036_854_775_808n]), "i64.wast:192", 1n);

// i64.wast:193
assert_return(() => call($1, "shr_s", [-9_223_372_036_854_775_808n, 63n]), "i64.wast:193", -1n);

// i64.wast:194
assert_return(() => call($1, "shr_s", [-1n, 64n]), "i64.wast:194", -1n);

// i64.wast:195
assert_return(() => call($1, "shr_s", [-1n, 65n]), "i64.wast:195", -1n);

// i64.wast:196
assert_return(() => call($1, "shr_s", [-1n, -1n]), "i64.wast:196", -1n);

// i64.wast:197
assert_return(() => call($1, "shr_s", [-1n, 9_223_372_036_854_775_807n]), "i64.wast:197", -1n);

// i64.wast:198
assert_return(() => call($1, "shr_s", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:198", -1n);

// i64.wast:200
assert_return(() => call($1, "shr_u", [1n, 1n]), "i64.wast:200", 0n);

// i64.wast:201
assert_return(() => call($1, "shr_u", [1n, 0n]), "i64.wast:201", 1n);

// i64.wast:202
assert_return(() => call($1, "shr_u", [-1n, 1n]), "i64.wast:202", 9_223_372_036_854_775_807n);

// i64.wast:203
assert_return(() => call($1, "shr_u", [9_223_372_036_854_775_807n, 1n]), "i64.wast:203", 4_611_686_018_427_387_903n);

// i64.wast:204
assert_return(() => call($1, "shr_u", [-9_223_372_036_854_775_808n, 1n]), "i64.wast:204", 4_611_686_018_427_387_904n);

// i64.wast:205
assert_return(() => call($1, "shr_u", [4_611_686_018_427_387_904n, 1n]), "i64.wast:205", 2_305_843_009_213_693_952n);

// i64.wast:206
assert_return(() => call($1, "shr_u", [1n, 64n]), "i64.wast:206", 1n);

// i64.wast:207
assert_return(() => call($1, "shr_u", [1n, 65n]), "i64.wast:207", 0n);

// i64.wast:208
assert_return(() => call($1, "shr_u", [1n, -1n]), "i64.wast:208", 0n);

// i64.wast:209
assert_return(() => call($1, "shr_u", [1n, 9_223_372_036_854_775_807n]), "i64.wast:209", 0n);

// i64.wast:210
assert_return(() => call($1, "shr_u", [1n, -9_223_372_036_854_775_808n]), "i64.wast:210", 1n);

// i64.wast:211
assert_return(() => call($1, "shr_u", [-9_223_372_036_854_775_808n, 63n]), "i64.wast:211", 1n);

// i64.wast:212
assert_return(() => call($1, "shr_u", [-1n, 64n]), "i64.wast:212", -1n);

// i64.wast:213
assert_return(() => call($1, "shr_u", [-1n, 65n]), "i64.wast:213", 9_223_372_036_854_775_807n);

// i64.wast:214
assert_return(() => call($1, "shr_u", [-1n, -1n]), "i64.wast:214", 1n);

// i64.wast:215
assert_return(() => call($1, "shr_u", [-1n, 9_223_372_036_854_775_807n]), "i64.wast:215", 1n);

// i64.wast:216
assert_return(() => call($1, "shr_u", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:216", -1n);

// i64.wast:218
assert_return(() => call($1, "rotl", [1n, 1n]), "i64.wast:218", 2n);

// i64.wast:219
assert_return(() => call($1, "rotl", [1n, 0n]), "i64.wast:219", 1n);

// i64.wast:220
assert_return(() => call($1, "rotl", [-1n, 1n]), "i64.wast:220", -1n);

// i64.wast:221
assert_return(() => call($1, "rotl", [1n, 64n]), "i64.wast:221", 1n);

// i64.wast:222
assert_return(() => call($1, "rotl", [-6_067_025_490_386_449_714n, 1n]), "i64.wast:222", 6_312_693_092_936_652_189n);

// i64.wast:223
assert_return(() => call($1, "rotl", [-144_115_184_384_868_352n, 4n]), "i64.wast:223", -2_305_842_950_157_893_617n);

// i64.wast:224
assert_return(() => call($1, "rotl", [-6_067_173_104_435_169_271n, 53n]), "i64.wast:224", 87_109_505_680_009_935n);

// i64.wast:225
assert_return(() => call($1, "rotl", [-6_066_028_401_059_725_156n, 63n]), "i64.wast:225", 6_190_357_836_324_913_230n);

// i64.wast:226
assert_return(() => call($1, "rotl", [-6_067_173_104_435_169_271n, 245n]), "i64.wast:226", 87_109_505_680_009_935n);

// i64.wast:227
assert_return(() => call($1, "rotl", [-6_067_067_139_002_042_359n, -19n]), "i64.wast:227", -3_530_481_836_149_793_302n);

// i64.wast:228
assert_return(() => call($1, "rotl", [-6_066_028_401_059_725_156n, -9_223_372_036_854_775_745n]), "i64.wast:228", 6_190_357_836_324_913_230n);

// i64.wast:229
assert_return(() => call($1, "rotl", [1n, 63n]), "i64.wast:229", -9_223_372_036_854_775_808n);

// i64.wast:230
assert_return(() => call($1, "rotl", [-9_223_372_036_854_775_808n, 1n]), "i64.wast:230", 1n);

// i64.wast:232
assert_return(() => call($1, "rotr", [1n, 1n]), "i64.wast:232", -9_223_372_036_854_775_808n);

// i64.wast:233
assert_return(() => call($1, "rotr", [1n, 0n]), "i64.wast:233", 1n);

// i64.wast:234
assert_return(() => call($1, "rotr", [-1n, 1n]), "i64.wast:234", -1n);

// i64.wast:235
assert_return(() => call($1, "rotr", [1n, 64n]), "i64.wast:235", 1n);

// i64.wast:236
assert_return(() => call($1, "rotr", [-6_067_025_490_386_449_714n, 1n]), "i64.wast:236", 6_189_859_291_661_550_951n);

// i64.wast:237
assert_return(() => call($1, "rotr", [-144_115_184_384_868_352n, 4n]), "i64.wast:237", 1_143_914_305_582_792_704n);

// i64.wast:238
assert_return(() => call($1, "rotr", [-6_067_173_104_435_169_271n, 53n]), "i64.wast:238", 7_534_987_797_011_123_550n);

// i64.wast:239
assert_return(() => call($1, "rotr", [-6_066_028_401_059_725_156n, 63n]), "i64.wast:239", 6_314_687_271_590_101_305n);

// i64.wast:240
assert_return(() => call($1, "rotr", [-6_067_173_104_435_169_271n, 245n]), "i64.wast:240", 7_534_987_797_011_123_550n);

// i64.wast:241
assert_return(() => call($1, "rotr", [-6_067_067_139_002_042_359n, -19n]), "i64.wast:241", -7_735_078_922_541_506_965n);

// i64.wast:242
assert_return(() => call($1, "rotr", [-6_066_028_401_059_725_156n, -9_223_372_036_854_775_745n]), "i64.wast:242", 6_314_687_271_590_101_305n);

// i64.wast:243
assert_return(() => call($1, "rotr", [1n, 63n]), "i64.wast:243", 2n);

// i64.wast:244
assert_return(() => call($1, "rotr", [-9_223_372_036_854_775_808n, 63n]), "i64.wast:244", 1n);

// i64.wast:246
assert_return(() => call($1, "clz", [-1n]), "i64.wast:246", 0n);

// i64.wast:247
assert_return(() => call($1, "clz", [0n]), "i64.wast:247", 64n);

// i64.wast:248
assert_return(() => call($1, "clz", [32_768n]), "i64.wast:248", 48n);

// i64.wast:249
assert_return(() => call($1, "clz", [255n]), "i64.wast:249", 56n);

// i64.wast:250
assert_return(() => call($1, "clz", [-9_223_372_036_854_775_808n]), "i64.wast:250", 0n);

// i64.wast:251
assert_return(() => call($1, "clz", [1n]), "i64.wast:251", 63n);

// i64.wast:252
assert_return(() => call($1, "clz", [2n]), "i64.wast:252", 62n);

// i64.wast:253
assert_return(() => call($1, "clz", [9_223_372_036_854_775_807n]), "i64.wast:253", 1n);

// i64.wast:255
assert_return(() => call($1, "ctz", [-1n]), "i64.wast:255", 0n);

// i64.wast:256
assert_return(() => call($1, "ctz", [0n]), "i64.wast:256", 64n);

// i64.wast:257
assert_return(() => call($1, "ctz", [32_768n]), "i64.wast:257", 15n);

// i64.wast:258
assert_return(() => call($1, "ctz", [65_536n]), "i64.wast:258", 16n);

// i64.wast:259
assert_return(() => call($1, "ctz", [-9_223_372_036_854_775_808n]), "i64.wast:259", 63n);

// i64.wast:260
assert_return(() => call($1, "ctz", [9_223_372_036_854_775_807n]), "i64.wast:260", 0n);

// i64.wast:262
assert_return(() => call($1, "popcnt", [-1n]), "i64.wast:262", 64n);

// i64.wast:263
assert_return(() => call($1, "popcnt", [0n]), "i64.wast:263", 0n);

// i64.wast:264
assert_return(() => call($1, "popcnt", [32_768n]), "i64.wast:264", 1n);

// i64.wast:265
assert_return(() => call($1, "popcnt", [-9_223_231_297_218_904_064n]), "i64.wast:265", 4n);

// i64.wast:266
assert_return(() => call($1, "popcnt", [9_223_372_036_854_775_807n]), "i64.wast:266", 63n);

// i64.wast:267
assert_return(() => call($1, "popcnt", [-6_148_914_692_668_172_971n]), "i64.wast:267", 32n);

// i64.wast:268
assert_return(() => call($1, "popcnt", [-7_378_697_629_197_489_494n]), "i64.wast:268", 32n);

// i64.wast:269
assert_return(() => call($1, "popcnt", [-2_401_053_088_876_216_593n]), "i64.wast:269", 48n);

// i64.wast:271
assert_return(() => call($1, "extend8_s", [0n]), "i64.wast:271", 0n);

// i64.wast:272
assert_return(() => call($1, "extend8_s", [127n]), "i64.wast:272", 127n);

// i64.wast:273
assert_return(() => call($1, "extend8_s", [128n]), "i64.wast:273", -128n);

// i64.wast:274
assert_return(() => call($1, "extend8_s", [255n]), "i64.wast:274", -1n);

// i64.wast:275
assert_return(() => call($1, "extend8_s", [81_985_529_216_486_656n]), "i64.wast:275", 0n);

// i64.wast:276
assert_return(() => call($1, "extend8_s", [-81_985_529_216_486_784n]), "i64.wast:276", -128n);

// i64.wast:277
assert_return(() => call($1, "extend8_s", [-1n]), "i64.wast:277", -1n);

// i64.wast:279
assert_return(() => call($1, "extend16_s", [0n]), "i64.wast:279", 0n);

// i64.wast:280
assert_return(() => call($1, "extend16_s", [32_767n]), "i64.wast:280", 32_767n);

// i64.wast:281
assert_return(() => call($1, "extend16_s", [32_768n]), "i64.wast:281", -32_768n);

// i64.wast:282
assert_return(() => call($1, "extend16_s", [65_535n]), "i64.wast:282", -1n);

// i64.wast:283
assert_return(() => call($1, "extend16_s", [1_311_768_467_463_733_248n]), "i64.wast:283", 0n);

// i64.wast:284
assert_return(() => call($1, "extend16_s", [-81_985_529_216_466_944n]), "i64.wast:284", -32_768n);

// i64.wast:285
assert_return(() => call($1, "extend16_s", [-1n]), "i64.wast:285", -1n);

// i64.wast:287
assert_return(() => call($1, "extend32_s", [0n]), "i64.wast:287", 0n);

// i64.wast:288
assert_return(() => call($1, "extend32_s", [32_767n]), "i64.wast:288", 32_767n);

// i64.wast:289
assert_return(() => call($1, "extend32_s", [32_768n]), "i64.wast:289", 32_768n);

// i64.wast:290
assert_return(() => call($1, "extend32_s", [65_535n]), "i64.wast:290", 65_535n);

// i64.wast:291
assert_return(() => call($1, "extend32_s", [2_147_483_647n]), "i64.wast:291", 2_147_483_647n);

// i64.wast:292
assert_return(() => call($1, "extend32_s", [2_147_483_648n]), "i64.wast:292", -2_147_483_648n);

// i64.wast:293
assert_return(() => call($1, "extend32_s", [4_294_967_295n]), "i64.wast:293", -1n);

// i64.wast:294
assert_return(() => call($1, "extend32_s", [81_985_526_906_748_928n]), "i64.wast:294", 0n);

// i64.wast:295
assert_return(() => call($1, "extend32_s", [-81_985_529_054_232_576n]), "i64.wast:295", -2_147_483_648n);

// i64.wast:296
assert_return(() => call($1, "extend32_s", [-1n]), "i64.wast:296", -1n);

// i64.wast:298
assert_return(() => call($1, "eqz", [0n]), "i64.wast:298", 1);

// i64.wast:299
assert_return(() => call($1, "eqz", [1n]), "i64.wast:299", 0);

// i64.wast:300
assert_return(() => call($1, "eqz", [-9_223_372_036_854_775_808n]), "i64.wast:300", 0);

// i64.wast:301
assert_return(() => call($1, "eqz", [9_223_372_036_854_775_807n]), "i64.wast:301", 0);

// i64.wast:302
assert_return(() => call($1, "eqz", [-1n]), "i64.wast:302", 0);

// i64.wast:304
assert_return(() => call($1, "eq", [0n, 0n]), "i64.wast:304", 1);

// i64.wast:305
assert_return(() => call($1, "eq", [1n, 1n]), "i64.wast:305", 1);

// i64.wast:306
assert_return(() => call($1, "eq", [-1n, 1n]), "i64.wast:306", 0);

// i64.wast:307
assert_return(() => call($1, "eq", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:307", 1);

// i64.wast:308
assert_return(() => call($1, "eq", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:308", 1);

// i64.wast:309
assert_return(() => call($1, "eq", [-1n, -1n]), "i64.wast:309", 1);

// i64.wast:310
assert_return(() => call($1, "eq", [1n, 0n]), "i64.wast:310", 0);

// i64.wast:311
assert_return(() => call($1, "eq", [0n, 1n]), "i64.wast:311", 0);

// i64.wast:312
assert_return(() => call($1, "eq", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:312", 0);

// i64.wast:313
assert_return(() => call($1, "eq", [0n, -9_223_372_036_854_775_808n]), "i64.wast:313", 0);

// i64.wast:314
assert_return(() => call($1, "eq", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:314", 0);

// i64.wast:315
assert_return(() => call($1, "eq", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:315", 0);

// i64.wast:316
assert_return(() => call($1, "eq", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:316", 0);

// i64.wast:317
assert_return(() => call($1, "eq", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:317", 0);

// i64.wast:319
assert_return(() => call($1, "ne", [0n, 0n]), "i64.wast:319", 0);

// i64.wast:320
assert_return(() => call($1, "ne", [1n, 1n]), "i64.wast:320", 0);

// i64.wast:321
assert_return(() => call($1, "ne", [-1n, 1n]), "i64.wast:321", 1);

// i64.wast:322
assert_return(() => call($1, "ne", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:322", 0);

// i64.wast:323
assert_return(() => call($1, "ne", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:323", 0);

// i64.wast:324
assert_return(() => call($1, "ne", [-1n, -1n]), "i64.wast:324", 0);

// i64.wast:325
assert_return(() => call($1, "ne", [1n, 0n]), "i64.wast:325", 1);

// i64.wast:326
assert_return(() => call($1, "ne", [0n, 1n]), "i64.wast:326", 1);

// i64.wast:327
assert_return(() => call($1, "ne", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:327", 1);

// i64.wast:328
assert_return(() => call($1, "ne", [0n, -9_223_372_036_854_775_808n]), "i64.wast:328", 1);

// i64.wast:329
assert_return(() => call($1, "ne", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:329", 1);

// i64.wast:330
assert_return(() => call($1, "ne", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:330", 1);

// i64.wast:331
assert_return(() => call($1, "ne", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:331", 1);

// i64.wast:332
assert_return(() => call($1, "ne", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:332", 1);

// i64.wast:334
assert_return(() => call($1, "lt_s", [0n, 0n]), "i64.wast:334", 0);

// i64.wast:335
assert_return(() => call($1, "lt_s", [1n, 1n]), "i64.wast:335", 0);

// i64.wast:336
assert_return(() => call($1, "lt_s", [-1n, 1n]), "i64.wast:336", 1);

// i64.wast:337
assert_return(() => call($1, "lt_s", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:337", 0);

// i64.wast:338
assert_return(() => call($1, "lt_s", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:338", 0);

// i64.wast:339
assert_return(() => call($1, "lt_s", [-1n, -1n]), "i64.wast:339", 0);

// i64.wast:340
assert_return(() => call($1, "lt_s", [1n, 0n]), "i64.wast:340", 0);

// i64.wast:341
assert_return(() => call($1, "lt_s", [0n, 1n]), "i64.wast:341", 1);

// i64.wast:342
assert_return(() => call($1, "lt_s", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:342", 1);

// i64.wast:343
assert_return(() => call($1, "lt_s", [0n, -9_223_372_036_854_775_808n]), "i64.wast:343", 0);

// i64.wast:344
assert_return(() => call($1, "lt_s", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:344", 1);

// i64.wast:345
assert_return(() => call($1, "lt_s", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:345", 0);

// i64.wast:346
assert_return(() => call($1, "lt_s", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:346", 1);

// i64.wast:347
assert_return(() => call($1, "lt_s", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:347", 0);

// i64.wast:349
assert_return(() => call($1, "lt_u", [0n, 0n]), "i64.wast:349", 0);

// i64.wast:350
assert_return(() => call($1, "lt_u", [1n, 1n]), "i64.wast:350", 0);

// i64.wast:351
assert_return(() => call($1, "lt_u", [-1n, 1n]), "i64.wast:351", 0);

// i64.wast:352
assert_return(() => call($1, "lt_u", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:352", 0);

// i64.wast:353
assert_return(() => call($1, "lt_u", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:353", 0);

// i64.wast:354
assert_return(() => call($1, "lt_u", [-1n, -1n]), "i64.wast:354", 0);

// i64.wast:355
assert_return(() => call($1, "lt_u", [1n, 0n]), "i64.wast:355", 0);

// i64.wast:356
assert_return(() => call($1, "lt_u", [0n, 1n]), "i64.wast:356", 1);

// i64.wast:357
assert_return(() => call($1, "lt_u", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:357", 0);

// i64.wast:358
assert_return(() => call($1, "lt_u", [0n, -9_223_372_036_854_775_808n]), "i64.wast:358", 1);

// i64.wast:359
assert_return(() => call($1, "lt_u", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:359", 1);

// i64.wast:360
assert_return(() => call($1, "lt_u", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:360", 0);

// i64.wast:361
assert_return(() => call($1, "lt_u", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:361", 0);

// i64.wast:362
assert_return(() => call($1, "lt_u", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:362", 1);

// i64.wast:364
assert_return(() => call($1, "le_s", [0n, 0n]), "i64.wast:364", 1);

// i64.wast:365
assert_return(() => call($1, "le_s", [1n, 1n]), "i64.wast:365", 1);

// i64.wast:366
assert_return(() => call($1, "le_s", [-1n, 1n]), "i64.wast:366", 1);

// i64.wast:367
assert_return(() => call($1, "le_s", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:367", 1);

// i64.wast:368
assert_return(() => call($1, "le_s", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:368", 1);

// i64.wast:369
assert_return(() => call($1, "le_s", [-1n, -1n]), "i64.wast:369", 1);

// i64.wast:370
assert_return(() => call($1, "le_s", [1n, 0n]), "i64.wast:370", 0);

// i64.wast:371
assert_return(() => call($1, "le_s", [0n, 1n]), "i64.wast:371", 1);

// i64.wast:372
assert_return(() => call($1, "le_s", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:372", 1);

// i64.wast:373
assert_return(() => call($1, "le_s", [0n, -9_223_372_036_854_775_808n]), "i64.wast:373", 0);

// i64.wast:374
assert_return(() => call($1, "le_s", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:374", 1);

// i64.wast:375
assert_return(() => call($1, "le_s", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:375", 0);

// i64.wast:376
assert_return(() => call($1, "le_s", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:376", 1);

// i64.wast:377
assert_return(() => call($1, "le_s", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:377", 0);

// i64.wast:379
assert_return(() => call($1, "le_u", [0n, 0n]), "i64.wast:379", 1);

// i64.wast:380
assert_return(() => call($1, "le_u", [1n, 1n]), "i64.wast:380", 1);

// i64.wast:381
assert_return(() => call($1, "le_u", [-1n, 1n]), "i64.wast:381", 0);

// i64.wast:382
assert_return(() => call($1, "le_u", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:382", 1);

// i64.wast:383
assert_return(() => call($1, "le_u", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:383", 1);

// i64.wast:384
assert_return(() => call($1, "le_u", [-1n, -1n]), "i64.wast:384", 1);

// i64.wast:385
assert_return(() => call($1, "le_u", [1n, 0n]), "i64.wast:385", 0);

// i64.wast:386
assert_return(() => call($1, "le_u", [0n, 1n]), "i64.wast:386", 1);

// i64.wast:387
assert_return(() => call($1, "le_u", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:387", 0);

// i64.wast:388
assert_return(() => call($1, "le_u", [0n, -9_223_372_036_854_775_808n]), "i64.wast:388", 1);

// i64.wast:389
assert_return(() => call($1, "le_u", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:389", 1);

// i64.wast:390
assert_return(() => call($1, "le_u", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:390", 0);

// i64.wast:391
assert_return(() => call($1, "le_u", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:391", 0);

// i64.wast:392
assert_return(() => call($1, "le_u", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:392", 1);

// i64.wast:394
assert_return(() => call($1, "gt_s", [0n, 0n]), "i64.wast:394", 0);

// i64.wast:395
assert_return(() => call($1, "gt_s", [1n, 1n]), "i64.wast:395", 0);

// i64.wast:396
assert_return(() => call($1, "gt_s", [-1n, 1n]), "i64.wast:396", 0);

// i64.wast:397
assert_return(() => call($1, "gt_s", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:397", 0);

// i64.wast:398
assert_return(() => call($1, "gt_s", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:398", 0);

// i64.wast:399
assert_return(() => call($1, "gt_s", [-1n, -1n]), "i64.wast:399", 0);

// i64.wast:400
assert_return(() => call($1, "gt_s", [1n, 0n]), "i64.wast:400", 1);

// i64.wast:401
assert_return(() => call($1, "gt_s", [0n, 1n]), "i64.wast:401", 0);

// i64.wast:402
assert_return(() => call($1, "gt_s", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:402", 0);

// i64.wast:403
assert_return(() => call($1, "gt_s", [0n, -9_223_372_036_854_775_808n]), "i64.wast:403", 1);

// i64.wast:404
assert_return(() => call($1, "gt_s", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:404", 0);

// i64.wast:405
assert_return(() => call($1, "gt_s", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:405", 1);

// i64.wast:406
assert_return(() => call($1, "gt_s", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:406", 0);

// i64.wast:407
assert_return(() => call($1, "gt_s", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:407", 1);

// i64.wast:409
assert_return(() => call($1, "gt_u", [0n, 0n]), "i64.wast:409", 0);

// i64.wast:410
assert_return(() => call($1, "gt_u", [1n, 1n]), "i64.wast:410", 0);

// i64.wast:411
assert_return(() => call($1, "gt_u", [-1n, 1n]), "i64.wast:411", 1);

// i64.wast:412
assert_return(() => call($1, "gt_u", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:412", 0);

// i64.wast:413
assert_return(() => call($1, "gt_u", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:413", 0);

// i64.wast:414
assert_return(() => call($1, "gt_u", [-1n, -1n]), "i64.wast:414", 0);

// i64.wast:415
assert_return(() => call($1, "gt_u", [1n, 0n]), "i64.wast:415", 1);

// i64.wast:416
assert_return(() => call($1, "gt_u", [0n, 1n]), "i64.wast:416", 0);

// i64.wast:417
assert_return(() => call($1, "gt_u", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:417", 1);

// i64.wast:418
assert_return(() => call($1, "gt_u", [0n, -9_223_372_036_854_775_808n]), "i64.wast:418", 0);

// i64.wast:419
assert_return(() => call($1, "gt_u", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:419", 0);

// i64.wast:420
assert_return(() => call($1, "gt_u", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:420", 1);

// i64.wast:421
assert_return(() => call($1, "gt_u", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:421", 1);

// i64.wast:422
assert_return(() => call($1, "gt_u", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:422", 0);

// i64.wast:424
assert_return(() => call($1, "ge_s", [0n, 0n]), "i64.wast:424", 1);

// i64.wast:425
assert_return(() => call($1, "ge_s", [1n, 1n]), "i64.wast:425", 1);

// i64.wast:426
assert_return(() => call($1, "ge_s", [-1n, 1n]), "i64.wast:426", 0);

// i64.wast:427
assert_return(() => call($1, "ge_s", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:427", 1);

// i64.wast:428
assert_return(() => call($1, "ge_s", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:428", 1);

// i64.wast:429
assert_return(() => call($1, "ge_s", [-1n, -1n]), "i64.wast:429", 1);

// i64.wast:430
assert_return(() => call($1, "ge_s", [1n, 0n]), "i64.wast:430", 1);

// i64.wast:431
assert_return(() => call($1, "ge_s", [0n, 1n]), "i64.wast:431", 0);

// i64.wast:432
assert_return(() => call($1, "ge_s", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:432", 0);

// i64.wast:433
assert_return(() => call($1, "ge_s", [0n, -9_223_372_036_854_775_808n]), "i64.wast:433", 1);

// i64.wast:434
assert_return(() => call($1, "ge_s", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:434", 0);

// i64.wast:435
assert_return(() => call($1, "ge_s", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:435", 1);

// i64.wast:436
assert_return(() => call($1, "ge_s", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:436", 0);

// i64.wast:437
assert_return(() => call($1, "ge_s", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:437", 1);

// i64.wast:439
assert_return(() => call($1, "ge_u", [0n, 0n]), "i64.wast:439", 1);

// i64.wast:440
assert_return(() => call($1, "ge_u", [1n, 1n]), "i64.wast:440", 1);

// i64.wast:441
assert_return(() => call($1, "ge_u", [-1n, 1n]), "i64.wast:441", 1);

// i64.wast:442
assert_return(() => call($1, "ge_u", [-9_223_372_036_854_775_808n, -9_223_372_036_854_775_808n]), "i64.wast:442", 1);

// i64.wast:443
assert_return(() => call($1, "ge_u", [9_223_372_036_854_775_807n, 9_223_372_036_854_775_807n]), "i64.wast:443", 1);

// i64.wast:444
assert_return(() => call($1, "ge_u", [-1n, -1n]), "i64.wast:444", 1);

// i64.wast:445
assert_return(() => call($1, "ge_u", [1n, 0n]), "i64.wast:445", 1);

// i64.wast:446
assert_return(() => call($1, "ge_u", [0n, 1n]), "i64.wast:446", 0);

// i64.wast:447
assert_return(() => call($1, "ge_u", [-9_223_372_036_854_775_808n, 0n]), "i64.wast:447", 1);

// i64.wast:448
assert_return(() => call($1, "ge_u", [0n, -9_223_372_036_854_775_808n]), "i64.wast:448", 0);

// i64.wast:449
assert_return(() => call($1, "ge_u", [-9_223_372_036_854_775_808n, -1n]), "i64.wast:449", 0);

// i64.wast:450
assert_return(() => call($1, "ge_u", [-1n, -9_223_372_036_854_775_808n]), "i64.wast:450", 1);

// i64.wast:451
assert_return(() => call($1, "ge_u", [-9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n]), "i64.wast:451", 1);

// i64.wast:452
assert_return(() => call($1, "ge_u", [9_223_372_036_854_775_807n, -9_223_372_036_854_775_808n]), "i64.wast:452", 0);

// i64.wast:457
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x7c\x0b", "i64.wast:457");

// i64.wast:458
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x83\x0b", "i64.wast:458");

// i64.wast:459
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x7f\x0b", "i64.wast:459");

// i64.wast:460
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x80\x0b", "i64.wast:460");

// i64.wast:461
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x7e\x0b", "i64.wast:461");

// i64.wast:462
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x84\x0b", "i64.wast:462");

// i64.wast:463
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x81\x0b", "i64.wast:463");

// i64.wast:464
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x82\x0b", "i64.wast:464");

// i64.wast:465
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x89\x0b", "i64.wast:465");

// i64.wast:466
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x8a\x0b", "i64.wast:466");

// i64.wast:467
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x86\x0b", "i64.wast:467");

// i64.wast:468
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x87\x0b", "i64.wast:468");

// i64.wast:469
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x88\x0b", "i64.wast:469");

// i64.wast:470
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x7d\x0b", "i64.wast:470");

// i64.wast:471
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x85\x0b", "i64.wast:471");

// i64.wast:472
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x41\x00\x50\x0b", "i64.wast:472");

// i64.wast:473
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x41\x00\x79\x0b", "i64.wast:473");

// i64.wast:474
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x41\x00\x7a\x0b", "i64.wast:474");

// i64.wast:475
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x41\x00\x7b\x0b", "i64.wast:475");

// i64.wast:476
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x51\x0b", "i64.wast:476");

// i64.wast:477
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x59\x0b", "i64.wast:477");

// i64.wast:478
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x5a\x0b", "i64.wast:478");

// i64.wast:479
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x55\x0b", "i64.wast:479");

// i64.wast:480
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x56\x0b", "i64.wast:480");

// i64.wast:481
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x57\x0b", "i64.wast:481");

// i64.wast:482
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x58\x0b", "i64.wast:482");

// i64.wast:483
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x53\x0b", "i64.wast:483");

// i64.wast:484
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x54\x0b", "i64.wast:484");

// i64.wast:485
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x43\x00\x00\x00\x00\x52\x0b", "i64.wast:485");

// i64.wast:487
assert_malformed("\x3c\x6d\x61\x6c\x66\x6f\x72\x6d\x65\x64\x20\x71\x75\x6f\x74\x65\x3e", "i64.wast:487");

// i64.wast:491
assert_malformed("\x3c\x6d\x61\x6c\x66\x6f\x72\x6d\x65\x64\x20\x71\x75\x6f\x74\x65\x3e", "i64.wast:491");
reinitializeRegistry();
})();
