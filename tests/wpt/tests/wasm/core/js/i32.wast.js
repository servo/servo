(function i32_wast_js() {

// i32.wast:3
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x02\x7f\x7f\x01\x7f\x60\x01\x7f\x01\x7f\x03\xa0\x80\x80\x80\x00\x1f\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x01\x01\x01\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x07\xde\x81\x80\x80\x00\x1f\x03\x61\x64\x64\x00\x00\x03\x73\x75\x62\x00\x01\x03\x6d\x75\x6c\x00\x02\x05\x64\x69\x76\x5f\x73\x00\x03\x05\x64\x69\x76\x5f\x75\x00\x04\x05\x72\x65\x6d\x5f\x73\x00\x05\x05\x72\x65\x6d\x5f\x75\x00\x06\x03\x61\x6e\x64\x00\x07\x02\x6f\x72\x00\x08\x03\x78\x6f\x72\x00\x09\x03\x73\x68\x6c\x00\x0a\x05\x73\x68\x72\x5f\x73\x00\x0b\x05\x73\x68\x72\x5f\x75\x00\x0c\x04\x72\x6f\x74\x6c\x00\x0d\x04\x72\x6f\x74\x72\x00\x0e\x03\x63\x6c\x7a\x00\x0f\x03\x63\x74\x7a\x00\x10\x06\x70\x6f\x70\x63\x6e\x74\x00\x11\x09\x65\x78\x74\x65\x6e\x64\x38\x5f\x73\x00\x12\x0a\x65\x78\x74\x65\x6e\x64\x31\x36\x5f\x73\x00\x13\x03\x65\x71\x7a\x00\x14\x02\x65\x71\x00\x15\x02\x6e\x65\x00\x16\x04\x6c\x74\x5f\x73\x00\x17\x04\x6c\x74\x5f\x75\x00\x18\x04\x6c\x65\x5f\x73\x00\x19\x04\x6c\x65\x5f\x75\x00\x1a\x04\x67\x74\x5f\x73\x00\x1b\x04\x67\x74\x5f\x75\x00\x1c\x04\x67\x65\x5f\x73\x00\x1d\x04\x67\x65\x5f\x75\x00\x1e\x0a\xe9\x82\x80\x80\x00\x1f\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x6a\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x6b\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x6c\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x6d\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x6e\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x6f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x70\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x71\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x72\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x73\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x74\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x75\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x76\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x77\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x78\x0b\x85\x80\x80\x80\x00\x00\x20\x00\x67\x0b\x85\x80\x80\x80\x00\x00\x20\x00\x68\x0b\x85\x80\x80\x80\x00\x00\x20\x00\x69\x0b\x85\x80\x80\x80\x00\x00\x20\x00\xc0\x0b\x85\x80\x80\x80\x00\x00\x20\x00\xc1\x0b\x85\x80\x80\x80\x00\x00\x20\x00\x45\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x46\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x47\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x48\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x49\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x4c\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x4d\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x4a\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x4b\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x4e\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x20\x01\x4f\x0b", "i32.wast:3");

// i32.wast:3
let $1 = instance($$1);

// i32.wast:37
assert_return(() => call($1, "add", [1, 1]), "i32.wast:37", 2);

// i32.wast:38
assert_return(() => call($1, "add", [1, 0]), "i32.wast:38", 1);

// i32.wast:39
assert_return(() => call($1, "add", [-1, -1]), "i32.wast:39", -2);

// i32.wast:40
assert_return(() => call($1, "add", [-1, 1]), "i32.wast:40", 0);

// i32.wast:41
assert_return(() => call($1, "add", [2_147_483_647, 1]), "i32.wast:41", -2_147_483_648);

// i32.wast:42
assert_return(() => call($1, "add", [-2_147_483_648, -1]), "i32.wast:42", 2_147_483_647);

// i32.wast:43
assert_return(() => call($1, "add", [-2_147_483_648, -2_147_483_648]), "i32.wast:43", 0);

// i32.wast:44
assert_return(() => call($1, "add", [1_073_741_823, 1]), "i32.wast:44", 1_073_741_824);

// i32.wast:46
assert_return(() => call($1, "sub", [1, 1]), "i32.wast:46", 0);

// i32.wast:47
assert_return(() => call($1, "sub", [1, 0]), "i32.wast:47", 1);

// i32.wast:48
assert_return(() => call($1, "sub", [-1, -1]), "i32.wast:48", 0);

// i32.wast:49
assert_return(() => call($1, "sub", [2_147_483_647, -1]), "i32.wast:49", -2_147_483_648);

// i32.wast:50
assert_return(() => call($1, "sub", [-2_147_483_648, 1]), "i32.wast:50", 2_147_483_647);

// i32.wast:51
assert_return(() => call($1, "sub", [-2_147_483_648, -2_147_483_648]), "i32.wast:51", 0);

// i32.wast:52
assert_return(() => call($1, "sub", [1_073_741_823, -1]), "i32.wast:52", 1_073_741_824);

// i32.wast:54
assert_return(() => call($1, "mul", [1, 1]), "i32.wast:54", 1);

// i32.wast:55
assert_return(() => call($1, "mul", [1, 0]), "i32.wast:55", 0);

// i32.wast:56
assert_return(() => call($1, "mul", [-1, -1]), "i32.wast:56", 1);

// i32.wast:57
assert_return(() => call($1, "mul", [268_435_456, 4_096]), "i32.wast:57", 0);

// i32.wast:58
assert_return(() => call($1, "mul", [-2_147_483_648, 0]), "i32.wast:58", 0);

// i32.wast:59
assert_return(() => call($1, "mul", [-2_147_483_648, -1]), "i32.wast:59", -2_147_483_648);

// i32.wast:60
assert_return(() => call($1, "mul", [2_147_483_647, -1]), "i32.wast:60", -2_147_483_647);

// i32.wast:61
assert_return(() => call($1, "mul", [19_088_743, 1_985_229_328]), "i32.wast:61", 898_528_368);

// i32.wast:62
assert_return(() => call($1, "mul", [2_147_483_647, 2_147_483_647]), "i32.wast:62", 1);

// i32.wast:64
assert_trap(() => call($1, "div_s", [1, 0]), "i32.wast:64");

// i32.wast:65
assert_trap(() => call($1, "div_s", [0, 0]), "i32.wast:65");

// i32.wast:66
assert_trap(() => call($1, "div_s", [-2_147_483_648, -1]), "i32.wast:66");

// i32.wast:67
assert_trap(() => call($1, "div_s", [-2_147_483_648, 0]), "i32.wast:67");

// i32.wast:68
assert_return(() => call($1, "div_s", [1, 1]), "i32.wast:68", 1);

// i32.wast:69
assert_return(() => call($1, "div_s", [0, 1]), "i32.wast:69", 0);

// i32.wast:70
assert_return(() => call($1, "div_s", [0, -1]), "i32.wast:70", 0);

// i32.wast:71
assert_return(() => call($1, "div_s", [-1, -1]), "i32.wast:71", 1);

// i32.wast:72
assert_return(() => call($1, "div_s", [-2_147_483_648, 2]), "i32.wast:72", -1_073_741_824);

// i32.wast:73
assert_return(() => call($1, "div_s", [-2_147_483_647, 1_000]), "i32.wast:73", -2_147_483);

// i32.wast:74
assert_return(() => call($1, "div_s", [5, 2]), "i32.wast:74", 2);

// i32.wast:75
assert_return(() => call($1, "div_s", [-5, 2]), "i32.wast:75", -2);

// i32.wast:76
assert_return(() => call($1, "div_s", [5, -2]), "i32.wast:76", -2);

// i32.wast:77
assert_return(() => call($1, "div_s", [-5, -2]), "i32.wast:77", 2);

// i32.wast:78
assert_return(() => call($1, "div_s", [7, 3]), "i32.wast:78", 2);

// i32.wast:79
assert_return(() => call($1, "div_s", [-7, 3]), "i32.wast:79", -2);

// i32.wast:80
assert_return(() => call($1, "div_s", [7, -3]), "i32.wast:80", -2);

// i32.wast:81
assert_return(() => call($1, "div_s", [-7, -3]), "i32.wast:81", 2);

// i32.wast:82
assert_return(() => call($1, "div_s", [11, 5]), "i32.wast:82", 2);

// i32.wast:83
assert_return(() => call($1, "div_s", [17, 7]), "i32.wast:83", 2);

// i32.wast:85
assert_trap(() => call($1, "div_u", [1, 0]), "i32.wast:85");

// i32.wast:86
assert_trap(() => call($1, "div_u", [0, 0]), "i32.wast:86");

// i32.wast:87
assert_return(() => call($1, "div_u", [1, 1]), "i32.wast:87", 1);

// i32.wast:88
assert_return(() => call($1, "div_u", [0, 1]), "i32.wast:88", 0);

// i32.wast:89
assert_return(() => call($1, "div_u", [-1, -1]), "i32.wast:89", 1);

// i32.wast:90
assert_return(() => call($1, "div_u", [-2_147_483_648, -1]), "i32.wast:90", 0);

// i32.wast:91
assert_return(() => call($1, "div_u", [-2_147_483_648, 2]), "i32.wast:91", 1_073_741_824);

// i32.wast:92
assert_return(() => call($1, "div_u", [-1_880_092_688, 65_537]), "i32.wast:92", 36_847);

// i32.wast:93
assert_return(() => call($1, "div_u", [-2_147_483_647, 1_000]), "i32.wast:93", 2_147_483);

// i32.wast:94
assert_return(() => call($1, "div_u", [5, 2]), "i32.wast:94", 2);

// i32.wast:95
assert_return(() => call($1, "div_u", [-5, 2]), "i32.wast:95", 2_147_483_645);

// i32.wast:96
assert_return(() => call($1, "div_u", [5, -2]), "i32.wast:96", 0);

// i32.wast:97
assert_return(() => call($1, "div_u", [-5, -2]), "i32.wast:97", 0);

// i32.wast:98
assert_return(() => call($1, "div_u", [7, 3]), "i32.wast:98", 2);

// i32.wast:99
assert_return(() => call($1, "div_u", [11, 5]), "i32.wast:99", 2);

// i32.wast:100
assert_return(() => call($1, "div_u", [17, 7]), "i32.wast:100", 2);

// i32.wast:102
assert_trap(() => call($1, "rem_s", [1, 0]), "i32.wast:102");

// i32.wast:103
assert_trap(() => call($1, "rem_s", [0, 0]), "i32.wast:103");

// i32.wast:104
assert_return(() => call($1, "rem_s", [2_147_483_647, -1]), "i32.wast:104", 0);

// i32.wast:105
assert_return(() => call($1, "rem_s", [1, 1]), "i32.wast:105", 0);

// i32.wast:106
assert_return(() => call($1, "rem_s", [0, 1]), "i32.wast:106", 0);

// i32.wast:107
assert_return(() => call($1, "rem_s", [0, -1]), "i32.wast:107", 0);

// i32.wast:108
assert_return(() => call($1, "rem_s", [-1, -1]), "i32.wast:108", 0);

// i32.wast:109
assert_return(() => call($1, "rem_s", [-2_147_483_648, -1]), "i32.wast:109", 0);

// i32.wast:110
assert_return(() => call($1, "rem_s", [-2_147_483_648, 2]), "i32.wast:110", 0);

// i32.wast:111
assert_return(() => call($1, "rem_s", [-2_147_483_647, 1_000]), "i32.wast:111", -647);

// i32.wast:112
assert_return(() => call($1, "rem_s", [5, 2]), "i32.wast:112", 1);

// i32.wast:113
assert_return(() => call($1, "rem_s", [-5, 2]), "i32.wast:113", -1);

// i32.wast:114
assert_return(() => call($1, "rem_s", [5, -2]), "i32.wast:114", 1);

// i32.wast:115
assert_return(() => call($1, "rem_s", [-5, -2]), "i32.wast:115", -1);

// i32.wast:116
assert_return(() => call($1, "rem_s", [7, 3]), "i32.wast:116", 1);

// i32.wast:117
assert_return(() => call($1, "rem_s", [-7, 3]), "i32.wast:117", -1);

// i32.wast:118
assert_return(() => call($1, "rem_s", [7, -3]), "i32.wast:118", 1);

// i32.wast:119
assert_return(() => call($1, "rem_s", [-7, -3]), "i32.wast:119", -1);

// i32.wast:120
assert_return(() => call($1, "rem_s", [11, 5]), "i32.wast:120", 1);

// i32.wast:121
assert_return(() => call($1, "rem_s", [17, 7]), "i32.wast:121", 3);

// i32.wast:123
assert_trap(() => call($1, "rem_u", [1, 0]), "i32.wast:123");

// i32.wast:124
assert_trap(() => call($1, "rem_u", [0, 0]), "i32.wast:124");

// i32.wast:125
assert_return(() => call($1, "rem_u", [1, 1]), "i32.wast:125", 0);

// i32.wast:126
assert_return(() => call($1, "rem_u", [0, 1]), "i32.wast:126", 0);

// i32.wast:127
assert_return(() => call($1, "rem_u", [-1, -1]), "i32.wast:127", 0);

// i32.wast:128
assert_return(() => call($1, "rem_u", [-2_147_483_648, -1]), "i32.wast:128", -2_147_483_648);

// i32.wast:129
assert_return(() => call($1, "rem_u", [-2_147_483_648, 2]), "i32.wast:129", 0);

// i32.wast:130
assert_return(() => call($1, "rem_u", [-1_880_092_688, 65_537]), "i32.wast:130", 32_769);

// i32.wast:131
assert_return(() => call($1, "rem_u", [-2_147_483_647, 1_000]), "i32.wast:131", 649);

// i32.wast:132
assert_return(() => call($1, "rem_u", [5, 2]), "i32.wast:132", 1);

// i32.wast:133
assert_return(() => call($1, "rem_u", [-5, 2]), "i32.wast:133", 1);

// i32.wast:134
assert_return(() => call($1, "rem_u", [5, -2]), "i32.wast:134", 5);

// i32.wast:135
assert_return(() => call($1, "rem_u", [-5, -2]), "i32.wast:135", -5);

// i32.wast:136
assert_return(() => call($1, "rem_u", [7, 3]), "i32.wast:136", 1);

// i32.wast:137
assert_return(() => call($1, "rem_u", [11, 5]), "i32.wast:137", 1);

// i32.wast:138
assert_return(() => call($1, "rem_u", [17, 7]), "i32.wast:138", 3);

// i32.wast:140
assert_return(() => call($1, "and", [1, 0]), "i32.wast:140", 0);

// i32.wast:141
assert_return(() => call($1, "and", [0, 1]), "i32.wast:141", 0);

// i32.wast:142
assert_return(() => call($1, "and", [1, 1]), "i32.wast:142", 1);

// i32.wast:143
assert_return(() => call($1, "and", [0, 0]), "i32.wast:143", 0);

// i32.wast:144
assert_return(() => call($1, "and", [2_147_483_647, -2_147_483_648]), "i32.wast:144", 0);

// i32.wast:145
assert_return(() => call($1, "and", [2_147_483_647, -1]), "i32.wast:145", 2_147_483_647);

// i32.wast:146
assert_return(() => call($1, "and", [-252_641_281, -3_856]), "i32.wast:146", -252_645_136);

// i32.wast:147
assert_return(() => call($1, "and", [-1, -1]), "i32.wast:147", -1);

// i32.wast:149
assert_return(() => call($1, "or", [1, 0]), "i32.wast:149", 1);

// i32.wast:150
assert_return(() => call($1, "or", [0, 1]), "i32.wast:150", 1);

// i32.wast:151
assert_return(() => call($1, "or", [1, 1]), "i32.wast:151", 1);

// i32.wast:152
assert_return(() => call($1, "or", [0, 0]), "i32.wast:152", 0);

// i32.wast:153
assert_return(() => call($1, "or", [2_147_483_647, -2_147_483_648]), "i32.wast:153", -1);

// i32.wast:154
assert_return(() => call($1, "or", [-2_147_483_648, 0]), "i32.wast:154", -2_147_483_648);

// i32.wast:155
assert_return(() => call($1, "or", [-252_641_281, -3_856]), "i32.wast:155", -1);

// i32.wast:156
assert_return(() => call($1, "or", [-1, -1]), "i32.wast:156", -1);

// i32.wast:158
assert_return(() => call($1, "xor", [1, 0]), "i32.wast:158", 1);

// i32.wast:159
assert_return(() => call($1, "xor", [0, 1]), "i32.wast:159", 1);

// i32.wast:160
assert_return(() => call($1, "xor", [1, 1]), "i32.wast:160", 0);

// i32.wast:161
assert_return(() => call($1, "xor", [0, 0]), "i32.wast:161", 0);

// i32.wast:162
assert_return(() => call($1, "xor", [2_147_483_647, -2_147_483_648]), "i32.wast:162", -1);

// i32.wast:163
assert_return(() => call($1, "xor", [-2_147_483_648, 0]), "i32.wast:163", -2_147_483_648);

// i32.wast:164
assert_return(() => call($1, "xor", [-1, -2_147_483_648]), "i32.wast:164", 2_147_483_647);

// i32.wast:165
assert_return(() => call($1, "xor", [-1, 2_147_483_647]), "i32.wast:165", -2_147_483_648);

// i32.wast:166
assert_return(() => call($1, "xor", [-252_641_281, -3_856]), "i32.wast:166", 252_645_135);

// i32.wast:167
assert_return(() => call($1, "xor", [-1, -1]), "i32.wast:167", 0);

// i32.wast:169
assert_return(() => call($1, "shl", [1, 1]), "i32.wast:169", 2);

// i32.wast:170
assert_return(() => call($1, "shl", [1, 0]), "i32.wast:170", 1);

// i32.wast:171
assert_return(() => call($1, "shl", [2_147_483_647, 1]), "i32.wast:171", -2);

// i32.wast:172
assert_return(() => call($1, "shl", [-1, 1]), "i32.wast:172", -2);

// i32.wast:173
assert_return(() => call($1, "shl", [-2_147_483_648, 1]), "i32.wast:173", 0);

// i32.wast:174
assert_return(() => call($1, "shl", [1_073_741_824, 1]), "i32.wast:174", -2_147_483_648);

// i32.wast:175
assert_return(() => call($1, "shl", [1, 31]), "i32.wast:175", -2_147_483_648);

// i32.wast:176
assert_return(() => call($1, "shl", [1, 32]), "i32.wast:176", 1);

// i32.wast:177
assert_return(() => call($1, "shl", [1, 33]), "i32.wast:177", 2);

// i32.wast:178
assert_return(() => call($1, "shl", [1, -1]), "i32.wast:178", -2_147_483_648);

// i32.wast:179
assert_return(() => call($1, "shl", [1, 2_147_483_647]), "i32.wast:179", -2_147_483_648);

// i32.wast:181
assert_return(() => call($1, "shr_s", [1, 1]), "i32.wast:181", 0);

// i32.wast:182
assert_return(() => call($1, "shr_s", [1, 0]), "i32.wast:182", 1);

// i32.wast:183
assert_return(() => call($1, "shr_s", [-1, 1]), "i32.wast:183", -1);

// i32.wast:184
assert_return(() => call($1, "shr_s", [2_147_483_647, 1]), "i32.wast:184", 1_073_741_823);

// i32.wast:185
assert_return(() => call($1, "shr_s", [-2_147_483_648, 1]), "i32.wast:185", -1_073_741_824);

// i32.wast:186
assert_return(() => call($1, "shr_s", [1_073_741_824, 1]), "i32.wast:186", 536_870_912);

// i32.wast:187
assert_return(() => call($1, "shr_s", [1, 32]), "i32.wast:187", 1);

// i32.wast:188
assert_return(() => call($1, "shr_s", [1, 33]), "i32.wast:188", 0);

// i32.wast:189
assert_return(() => call($1, "shr_s", [1, -1]), "i32.wast:189", 0);

// i32.wast:190
assert_return(() => call($1, "shr_s", [1, 2_147_483_647]), "i32.wast:190", 0);

// i32.wast:191
assert_return(() => call($1, "shr_s", [1, -2_147_483_648]), "i32.wast:191", 1);

// i32.wast:192
assert_return(() => call($1, "shr_s", [-2_147_483_648, 31]), "i32.wast:192", -1);

// i32.wast:193
assert_return(() => call($1, "shr_s", [-1, 32]), "i32.wast:193", -1);

// i32.wast:194
assert_return(() => call($1, "shr_s", [-1, 33]), "i32.wast:194", -1);

// i32.wast:195
assert_return(() => call($1, "shr_s", [-1, -1]), "i32.wast:195", -1);

// i32.wast:196
assert_return(() => call($1, "shr_s", [-1, 2_147_483_647]), "i32.wast:196", -1);

// i32.wast:197
assert_return(() => call($1, "shr_s", [-1, -2_147_483_648]), "i32.wast:197", -1);

// i32.wast:199
assert_return(() => call($1, "shr_u", [1, 1]), "i32.wast:199", 0);

// i32.wast:200
assert_return(() => call($1, "shr_u", [1, 0]), "i32.wast:200", 1);

// i32.wast:201
assert_return(() => call($1, "shr_u", [-1, 1]), "i32.wast:201", 2_147_483_647);

// i32.wast:202
assert_return(() => call($1, "shr_u", [2_147_483_647, 1]), "i32.wast:202", 1_073_741_823);

// i32.wast:203
assert_return(() => call($1, "shr_u", [-2_147_483_648, 1]), "i32.wast:203", 1_073_741_824);

// i32.wast:204
assert_return(() => call($1, "shr_u", [1_073_741_824, 1]), "i32.wast:204", 536_870_912);

// i32.wast:205
assert_return(() => call($1, "shr_u", [1, 32]), "i32.wast:205", 1);

// i32.wast:206
assert_return(() => call($1, "shr_u", [1, 33]), "i32.wast:206", 0);

// i32.wast:207
assert_return(() => call($1, "shr_u", [1, -1]), "i32.wast:207", 0);

// i32.wast:208
assert_return(() => call($1, "shr_u", [1, 2_147_483_647]), "i32.wast:208", 0);

// i32.wast:209
assert_return(() => call($1, "shr_u", [1, -2_147_483_648]), "i32.wast:209", 1);

// i32.wast:210
assert_return(() => call($1, "shr_u", [-2_147_483_648, 31]), "i32.wast:210", 1);

// i32.wast:211
assert_return(() => call($1, "shr_u", [-1, 32]), "i32.wast:211", -1);

// i32.wast:212
assert_return(() => call($1, "shr_u", [-1, 33]), "i32.wast:212", 2_147_483_647);

// i32.wast:213
assert_return(() => call($1, "shr_u", [-1, -1]), "i32.wast:213", 1);

// i32.wast:214
assert_return(() => call($1, "shr_u", [-1, 2_147_483_647]), "i32.wast:214", 1);

// i32.wast:215
assert_return(() => call($1, "shr_u", [-1, -2_147_483_648]), "i32.wast:215", -1);

// i32.wast:217
assert_return(() => call($1, "rotl", [1, 1]), "i32.wast:217", 2);

// i32.wast:218
assert_return(() => call($1, "rotl", [1, 0]), "i32.wast:218", 1);

// i32.wast:219
assert_return(() => call($1, "rotl", [-1, 1]), "i32.wast:219", -1);

// i32.wast:220
assert_return(() => call($1, "rotl", [1, 32]), "i32.wast:220", 1);

// i32.wast:221
assert_return(() => call($1, "rotl", [-1_412_589_450, 1]), "i32.wast:221", 1_469_788_397);

// i32.wast:222
assert_return(() => call($1, "rotl", [-33_498_112, 4]), "i32.wast:222", -535_969_777);

// i32.wast:223
assert_return(() => call($1, "rotl", [-1_329_474_845, 5]), "i32.wast:223", 406_477_942);

// i32.wast:224
assert_return(() => call($1, "rotl", [32_768, 37]), "i32.wast:224", 1_048_576);

// i32.wast:225
assert_return(() => call($1, "rotl", [-1_329_474_845, 65_285]), "i32.wast:225", 406_477_942);

// i32.wast:226
assert_return(() => call($1, "rotl", [1_989_852_383, -19]), "i32.wast:226", 1_469_837_011);

// i32.wast:227
assert_return(() => call($1, "rotl", [1_989_852_383, -2_147_483_635]), "i32.wast:227", 1_469_837_011);

// i32.wast:228
assert_return(() => call($1, "rotl", [1, 31]), "i32.wast:228", -2_147_483_648);

// i32.wast:229
assert_return(() => call($1, "rotl", [-2_147_483_648, 1]), "i32.wast:229", 1);

// i32.wast:231
assert_return(() => call($1, "rotr", [1, 1]), "i32.wast:231", -2_147_483_648);

// i32.wast:232
assert_return(() => call($1, "rotr", [1, 0]), "i32.wast:232", 1);

// i32.wast:233
assert_return(() => call($1, "rotr", [-1, 1]), "i32.wast:233", -1);

// i32.wast:234
assert_return(() => call($1, "rotr", [1, 32]), "i32.wast:234", 1);

// i32.wast:235
assert_return(() => call($1, "rotr", [-16_724_992, 1]), "i32.wast:235", 2_139_121_152);

// i32.wast:236
assert_return(() => call($1, "rotr", [524_288, 4]), "i32.wast:236", 32_768);

// i32.wast:237
assert_return(() => call($1, "rotr", [-1_329_474_845, 5]), "i32.wast:237", 495_324_823);

// i32.wast:238
assert_return(() => call($1, "rotr", [32_768, 37]), "i32.wast:238", 1_024);

// i32.wast:239
assert_return(() => call($1, "rotr", [-1_329_474_845, 65_285]), "i32.wast:239", 495_324_823);

// i32.wast:240
assert_return(() => call($1, "rotr", [1_989_852_383, -19]), "i32.wast:240", -419_711_787);

// i32.wast:241
assert_return(() => call($1, "rotr", [1_989_852_383, -2_147_483_635]), "i32.wast:241", -419_711_787);

// i32.wast:242
assert_return(() => call($1, "rotr", [1, 31]), "i32.wast:242", 2);

// i32.wast:243
assert_return(() => call($1, "rotr", [-2_147_483_648, 31]), "i32.wast:243", 1);

// i32.wast:245
assert_return(() => call($1, "clz", [-1]), "i32.wast:245", 0);

// i32.wast:246
assert_return(() => call($1, "clz", [0]), "i32.wast:246", 32);

// i32.wast:247
assert_return(() => call($1, "clz", [32_768]), "i32.wast:247", 16);

// i32.wast:248
assert_return(() => call($1, "clz", [255]), "i32.wast:248", 24);

// i32.wast:249
assert_return(() => call($1, "clz", [-2_147_483_648]), "i32.wast:249", 0);

// i32.wast:250
assert_return(() => call($1, "clz", [1]), "i32.wast:250", 31);

// i32.wast:251
assert_return(() => call($1, "clz", [2]), "i32.wast:251", 30);

// i32.wast:252
assert_return(() => call($1, "clz", [2_147_483_647]), "i32.wast:252", 1);

// i32.wast:254
assert_return(() => call($1, "ctz", [-1]), "i32.wast:254", 0);

// i32.wast:255
assert_return(() => call($1, "ctz", [0]), "i32.wast:255", 32);

// i32.wast:256
assert_return(() => call($1, "ctz", [32_768]), "i32.wast:256", 15);

// i32.wast:257
assert_return(() => call($1, "ctz", [65_536]), "i32.wast:257", 16);

// i32.wast:258
assert_return(() => call($1, "ctz", [-2_147_483_648]), "i32.wast:258", 31);

// i32.wast:259
assert_return(() => call($1, "ctz", [2_147_483_647]), "i32.wast:259", 0);

// i32.wast:261
assert_return(() => call($1, "popcnt", [-1]), "i32.wast:261", 32);

// i32.wast:262
assert_return(() => call($1, "popcnt", [0]), "i32.wast:262", 0);

// i32.wast:263
assert_return(() => call($1, "popcnt", [32_768]), "i32.wast:263", 1);

// i32.wast:264
assert_return(() => call($1, "popcnt", [-2_147_450_880]), "i32.wast:264", 2);

// i32.wast:265
assert_return(() => call($1, "popcnt", [2_147_483_647]), "i32.wast:265", 31);

// i32.wast:266
assert_return(() => call($1, "popcnt", [-1_431_655_766]), "i32.wast:266", 16);

// i32.wast:267
assert_return(() => call($1, "popcnt", [1_431_655_765]), "i32.wast:267", 16);

// i32.wast:268
assert_return(() => call($1, "popcnt", [-559_038_737]), "i32.wast:268", 24);

// i32.wast:270
assert_return(() => call($1, "extend8_s", [0]), "i32.wast:270", 0);

// i32.wast:271
assert_return(() => call($1, "extend8_s", [127]), "i32.wast:271", 127);

// i32.wast:272
assert_return(() => call($1, "extend8_s", [128]), "i32.wast:272", -128);

// i32.wast:273
assert_return(() => call($1, "extend8_s", [255]), "i32.wast:273", -1);

// i32.wast:274
assert_return(() => call($1, "extend8_s", [19_088_640]), "i32.wast:274", 0);

// i32.wast:275
assert_return(() => call($1, "extend8_s", [-19_088_768]), "i32.wast:275", -128);

// i32.wast:276
assert_return(() => call($1, "extend8_s", [-1]), "i32.wast:276", -1);

// i32.wast:278
assert_return(() => call($1, "extend16_s", [0]), "i32.wast:278", 0);

// i32.wast:279
assert_return(() => call($1, "extend16_s", [32_767]), "i32.wast:279", 32_767);

// i32.wast:280
assert_return(() => call($1, "extend16_s", [32_768]), "i32.wast:280", -32_768);

// i32.wast:281
assert_return(() => call($1, "extend16_s", [65_535]), "i32.wast:281", -1);

// i32.wast:282
assert_return(() => call($1, "extend16_s", [19_070_976]), "i32.wast:282", 0);

// i32.wast:283
assert_return(() => call($1, "extend16_s", [-19_103_744]), "i32.wast:283", -32_768);

// i32.wast:284
assert_return(() => call($1, "extend16_s", [-1]), "i32.wast:284", -1);

// i32.wast:286
assert_return(() => call($1, "eqz", [0]), "i32.wast:286", 1);

// i32.wast:287
assert_return(() => call($1, "eqz", [1]), "i32.wast:287", 0);

// i32.wast:288
assert_return(() => call($1, "eqz", [-2_147_483_648]), "i32.wast:288", 0);

// i32.wast:289
assert_return(() => call($1, "eqz", [2_147_483_647]), "i32.wast:289", 0);

// i32.wast:290
assert_return(() => call($1, "eqz", [-1]), "i32.wast:290", 0);

// i32.wast:292
assert_return(() => call($1, "eq", [0, 0]), "i32.wast:292", 1);

// i32.wast:293
assert_return(() => call($1, "eq", [1, 1]), "i32.wast:293", 1);

// i32.wast:294
assert_return(() => call($1, "eq", [-1, 1]), "i32.wast:294", 0);

// i32.wast:295
assert_return(() => call($1, "eq", [-2_147_483_648, -2_147_483_648]), "i32.wast:295", 1);

// i32.wast:296
assert_return(() => call($1, "eq", [2_147_483_647, 2_147_483_647]), "i32.wast:296", 1);

// i32.wast:297
assert_return(() => call($1, "eq", [-1, -1]), "i32.wast:297", 1);

// i32.wast:298
assert_return(() => call($1, "eq", [1, 0]), "i32.wast:298", 0);

// i32.wast:299
assert_return(() => call($1, "eq", [0, 1]), "i32.wast:299", 0);

// i32.wast:300
assert_return(() => call($1, "eq", [-2_147_483_648, 0]), "i32.wast:300", 0);

// i32.wast:301
assert_return(() => call($1, "eq", [0, -2_147_483_648]), "i32.wast:301", 0);

// i32.wast:302
assert_return(() => call($1, "eq", [-2_147_483_648, -1]), "i32.wast:302", 0);

// i32.wast:303
assert_return(() => call($1, "eq", [-1, -2_147_483_648]), "i32.wast:303", 0);

// i32.wast:304
assert_return(() => call($1, "eq", [-2_147_483_648, 2_147_483_647]), "i32.wast:304", 0);

// i32.wast:305
assert_return(() => call($1, "eq", [2_147_483_647, -2_147_483_648]), "i32.wast:305", 0);

// i32.wast:307
assert_return(() => call($1, "ne", [0, 0]), "i32.wast:307", 0);

// i32.wast:308
assert_return(() => call($1, "ne", [1, 1]), "i32.wast:308", 0);

// i32.wast:309
assert_return(() => call($1, "ne", [-1, 1]), "i32.wast:309", 1);

// i32.wast:310
assert_return(() => call($1, "ne", [-2_147_483_648, -2_147_483_648]), "i32.wast:310", 0);

// i32.wast:311
assert_return(() => call($1, "ne", [2_147_483_647, 2_147_483_647]), "i32.wast:311", 0);

// i32.wast:312
assert_return(() => call($1, "ne", [-1, -1]), "i32.wast:312", 0);

// i32.wast:313
assert_return(() => call($1, "ne", [1, 0]), "i32.wast:313", 1);

// i32.wast:314
assert_return(() => call($1, "ne", [0, 1]), "i32.wast:314", 1);

// i32.wast:315
assert_return(() => call($1, "ne", [-2_147_483_648, 0]), "i32.wast:315", 1);

// i32.wast:316
assert_return(() => call($1, "ne", [0, -2_147_483_648]), "i32.wast:316", 1);

// i32.wast:317
assert_return(() => call($1, "ne", [-2_147_483_648, -1]), "i32.wast:317", 1);

// i32.wast:318
assert_return(() => call($1, "ne", [-1, -2_147_483_648]), "i32.wast:318", 1);

// i32.wast:319
assert_return(() => call($1, "ne", [-2_147_483_648, 2_147_483_647]), "i32.wast:319", 1);

// i32.wast:320
assert_return(() => call($1, "ne", [2_147_483_647, -2_147_483_648]), "i32.wast:320", 1);

// i32.wast:322
assert_return(() => call($1, "lt_s", [0, 0]), "i32.wast:322", 0);

// i32.wast:323
assert_return(() => call($1, "lt_s", [1, 1]), "i32.wast:323", 0);

// i32.wast:324
assert_return(() => call($1, "lt_s", [-1, 1]), "i32.wast:324", 1);

// i32.wast:325
assert_return(() => call($1, "lt_s", [-2_147_483_648, -2_147_483_648]), "i32.wast:325", 0);

// i32.wast:326
assert_return(() => call($1, "lt_s", [2_147_483_647, 2_147_483_647]), "i32.wast:326", 0);

// i32.wast:327
assert_return(() => call($1, "lt_s", [-1, -1]), "i32.wast:327", 0);

// i32.wast:328
assert_return(() => call($1, "lt_s", [1, 0]), "i32.wast:328", 0);

// i32.wast:329
assert_return(() => call($1, "lt_s", [0, 1]), "i32.wast:329", 1);

// i32.wast:330
assert_return(() => call($1, "lt_s", [-2_147_483_648, 0]), "i32.wast:330", 1);

// i32.wast:331
assert_return(() => call($1, "lt_s", [0, -2_147_483_648]), "i32.wast:331", 0);

// i32.wast:332
assert_return(() => call($1, "lt_s", [-2_147_483_648, -1]), "i32.wast:332", 1);

// i32.wast:333
assert_return(() => call($1, "lt_s", [-1, -2_147_483_648]), "i32.wast:333", 0);

// i32.wast:334
assert_return(() => call($1, "lt_s", [-2_147_483_648, 2_147_483_647]), "i32.wast:334", 1);

// i32.wast:335
assert_return(() => call($1, "lt_s", [2_147_483_647, -2_147_483_648]), "i32.wast:335", 0);

// i32.wast:337
assert_return(() => call($1, "lt_u", [0, 0]), "i32.wast:337", 0);

// i32.wast:338
assert_return(() => call($1, "lt_u", [1, 1]), "i32.wast:338", 0);

// i32.wast:339
assert_return(() => call($1, "lt_u", [-1, 1]), "i32.wast:339", 0);

// i32.wast:340
assert_return(() => call($1, "lt_u", [-2_147_483_648, -2_147_483_648]), "i32.wast:340", 0);

// i32.wast:341
assert_return(() => call($1, "lt_u", [2_147_483_647, 2_147_483_647]), "i32.wast:341", 0);

// i32.wast:342
assert_return(() => call($1, "lt_u", [-1, -1]), "i32.wast:342", 0);

// i32.wast:343
assert_return(() => call($1, "lt_u", [1, 0]), "i32.wast:343", 0);

// i32.wast:344
assert_return(() => call($1, "lt_u", [0, 1]), "i32.wast:344", 1);

// i32.wast:345
assert_return(() => call($1, "lt_u", [-2_147_483_648, 0]), "i32.wast:345", 0);

// i32.wast:346
assert_return(() => call($1, "lt_u", [0, -2_147_483_648]), "i32.wast:346", 1);

// i32.wast:347
assert_return(() => call($1, "lt_u", [-2_147_483_648, -1]), "i32.wast:347", 1);

// i32.wast:348
assert_return(() => call($1, "lt_u", [-1, -2_147_483_648]), "i32.wast:348", 0);

// i32.wast:349
assert_return(() => call($1, "lt_u", [-2_147_483_648, 2_147_483_647]), "i32.wast:349", 0);

// i32.wast:350
assert_return(() => call($1, "lt_u", [2_147_483_647, -2_147_483_648]), "i32.wast:350", 1);

// i32.wast:352
assert_return(() => call($1, "le_s", [0, 0]), "i32.wast:352", 1);

// i32.wast:353
assert_return(() => call($1, "le_s", [1, 1]), "i32.wast:353", 1);

// i32.wast:354
assert_return(() => call($1, "le_s", [-1, 1]), "i32.wast:354", 1);

// i32.wast:355
assert_return(() => call($1, "le_s", [-2_147_483_648, -2_147_483_648]), "i32.wast:355", 1);

// i32.wast:356
assert_return(() => call($1, "le_s", [2_147_483_647, 2_147_483_647]), "i32.wast:356", 1);

// i32.wast:357
assert_return(() => call($1, "le_s", [-1, -1]), "i32.wast:357", 1);

// i32.wast:358
assert_return(() => call($1, "le_s", [1, 0]), "i32.wast:358", 0);

// i32.wast:359
assert_return(() => call($1, "le_s", [0, 1]), "i32.wast:359", 1);

// i32.wast:360
assert_return(() => call($1, "le_s", [-2_147_483_648, 0]), "i32.wast:360", 1);

// i32.wast:361
assert_return(() => call($1, "le_s", [0, -2_147_483_648]), "i32.wast:361", 0);

// i32.wast:362
assert_return(() => call($1, "le_s", [-2_147_483_648, -1]), "i32.wast:362", 1);

// i32.wast:363
assert_return(() => call($1, "le_s", [-1, -2_147_483_648]), "i32.wast:363", 0);

// i32.wast:364
assert_return(() => call($1, "le_s", [-2_147_483_648, 2_147_483_647]), "i32.wast:364", 1);

// i32.wast:365
assert_return(() => call($1, "le_s", [2_147_483_647, -2_147_483_648]), "i32.wast:365", 0);

// i32.wast:367
assert_return(() => call($1, "le_u", [0, 0]), "i32.wast:367", 1);

// i32.wast:368
assert_return(() => call($1, "le_u", [1, 1]), "i32.wast:368", 1);

// i32.wast:369
assert_return(() => call($1, "le_u", [-1, 1]), "i32.wast:369", 0);

// i32.wast:370
assert_return(() => call($1, "le_u", [-2_147_483_648, -2_147_483_648]), "i32.wast:370", 1);

// i32.wast:371
assert_return(() => call($1, "le_u", [2_147_483_647, 2_147_483_647]), "i32.wast:371", 1);

// i32.wast:372
assert_return(() => call($1, "le_u", [-1, -1]), "i32.wast:372", 1);

// i32.wast:373
assert_return(() => call($1, "le_u", [1, 0]), "i32.wast:373", 0);

// i32.wast:374
assert_return(() => call($1, "le_u", [0, 1]), "i32.wast:374", 1);

// i32.wast:375
assert_return(() => call($1, "le_u", [-2_147_483_648, 0]), "i32.wast:375", 0);

// i32.wast:376
assert_return(() => call($1, "le_u", [0, -2_147_483_648]), "i32.wast:376", 1);

// i32.wast:377
assert_return(() => call($1, "le_u", [-2_147_483_648, -1]), "i32.wast:377", 1);

// i32.wast:378
assert_return(() => call($1, "le_u", [-1, -2_147_483_648]), "i32.wast:378", 0);

// i32.wast:379
assert_return(() => call($1, "le_u", [-2_147_483_648, 2_147_483_647]), "i32.wast:379", 0);

// i32.wast:380
assert_return(() => call($1, "le_u", [2_147_483_647, -2_147_483_648]), "i32.wast:380", 1);

// i32.wast:382
assert_return(() => call($1, "gt_s", [0, 0]), "i32.wast:382", 0);

// i32.wast:383
assert_return(() => call($1, "gt_s", [1, 1]), "i32.wast:383", 0);

// i32.wast:384
assert_return(() => call($1, "gt_s", [-1, 1]), "i32.wast:384", 0);

// i32.wast:385
assert_return(() => call($1, "gt_s", [-2_147_483_648, -2_147_483_648]), "i32.wast:385", 0);

// i32.wast:386
assert_return(() => call($1, "gt_s", [2_147_483_647, 2_147_483_647]), "i32.wast:386", 0);

// i32.wast:387
assert_return(() => call($1, "gt_s", [-1, -1]), "i32.wast:387", 0);

// i32.wast:388
assert_return(() => call($1, "gt_s", [1, 0]), "i32.wast:388", 1);

// i32.wast:389
assert_return(() => call($1, "gt_s", [0, 1]), "i32.wast:389", 0);

// i32.wast:390
assert_return(() => call($1, "gt_s", [-2_147_483_648, 0]), "i32.wast:390", 0);

// i32.wast:391
assert_return(() => call($1, "gt_s", [0, -2_147_483_648]), "i32.wast:391", 1);

// i32.wast:392
assert_return(() => call($1, "gt_s", [-2_147_483_648, -1]), "i32.wast:392", 0);

// i32.wast:393
assert_return(() => call($1, "gt_s", [-1, -2_147_483_648]), "i32.wast:393", 1);

// i32.wast:394
assert_return(() => call($1, "gt_s", [-2_147_483_648, 2_147_483_647]), "i32.wast:394", 0);

// i32.wast:395
assert_return(() => call($1, "gt_s", [2_147_483_647, -2_147_483_648]), "i32.wast:395", 1);

// i32.wast:397
assert_return(() => call($1, "gt_u", [0, 0]), "i32.wast:397", 0);

// i32.wast:398
assert_return(() => call($1, "gt_u", [1, 1]), "i32.wast:398", 0);

// i32.wast:399
assert_return(() => call($1, "gt_u", [-1, 1]), "i32.wast:399", 1);

// i32.wast:400
assert_return(() => call($1, "gt_u", [-2_147_483_648, -2_147_483_648]), "i32.wast:400", 0);

// i32.wast:401
assert_return(() => call($1, "gt_u", [2_147_483_647, 2_147_483_647]), "i32.wast:401", 0);

// i32.wast:402
assert_return(() => call($1, "gt_u", [-1, -1]), "i32.wast:402", 0);

// i32.wast:403
assert_return(() => call($1, "gt_u", [1, 0]), "i32.wast:403", 1);

// i32.wast:404
assert_return(() => call($1, "gt_u", [0, 1]), "i32.wast:404", 0);

// i32.wast:405
assert_return(() => call($1, "gt_u", [-2_147_483_648, 0]), "i32.wast:405", 1);

// i32.wast:406
assert_return(() => call($1, "gt_u", [0, -2_147_483_648]), "i32.wast:406", 0);

// i32.wast:407
assert_return(() => call($1, "gt_u", [-2_147_483_648, -1]), "i32.wast:407", 0);

// i32.wast:408
assert_return(() => call($1, "gt_u", [-1, -2_147_483_648]), "i32.wast:408", 1);

// i32.wast:409
assert_return(() => call($1, "gt_u", [-2_147_483_648, 2_147_483_647]), "i32.wast:409", 1);

// i32.wast:410
assert_return(() => call($1, "gt_u", [2_147_483_647, -2_147_483_648]), "i32.wast:410", 0);

// i32.wast:412
assert_return(() => call($1, "ge_s", [0, 0]), "i32.wast:412", 1);

// i32.wast:413
assert_return(() => call($1, "ge_s", [1, 1]), "i32.wast:413", 1);

// i32.wast:414
assert_return(() => call($1, "ge_s", [-1, 1]), "i32.wast:414", 0);

// i32.wast:415
assert_return(() => call($1, "ge_s", [-2_147_483_648, -2_147_483_648]), "i32.wast:415", 1);

// i32.wast:416
assert_return(() => call($1, "ge_s", [2_147_483_647, 2_147_483_647]), "i32.wast:416", 1);

// i32.wast:417
assert_return(() => call($1, "ge_s", [-1, -1]), "i32.wast:417", 1);

// i32.wast:418
assert_return(() => call($1, "ge_s", [1, 0]), "i32.wast:418", 1);

// i32.wast:419
assert_return(() => call($1, "ge_s", [0, 1]), "i32.wast:419", 0);

// i32.wast:420
assert_return(() => call($1, "ge_s", [-2_147_483_648, 0]), "i32.wast:420", 0);

// i32.wast:421
assert_return(() => call($1, "ge_s", [0, -2_147_483_648]), "i32.wast:421", 1);

// i32.wast:422
assert_return(() => call($1, "ge_s", [-2_147_483_648, -1]), "i32.wast:422", 0);

// i32.wast:423
assert_return(() => call($1, "ge_s", [-1, -2_147_483_648]), "i32.wast:423", 1);

// i32.wast:424
assert_return(() => call($1, "ge_s", [-2_147_483_648, 2_147_483_647]), "i32.wast:424", 0);

// i32.wast:425
assert_return(() => call($1, "ge_s", [2_147_483_647, -2_147_483_648]), "i32.wast:425", 1);

// i32.wast:427
assert_return(() => call($1, "ge_u", [0, 0]), "i32.wast:427", 1);

// i32.wast:428
assert_return(() => call($1, "ge_u", [1, 1]), "i32.wast:428", 1);

// i32.wast:429
assert_return(() => call($1, "ge_u", [-1, 1]), "i32.wast:429", 1);

// i32.wast:430
assert_return(() => call($1, "ge_u", [-2_147_483_648, -2_147_483_648]), "i32.wast:430", 1);

// i32.wast:431
assert_return(() => call($1, "ge_u", [2_147_483_647, 2_147_483_647]), "i32.wast:431", 1);

// i32.wast:432
assert_return(() => call($1, "ge_u", [-1, -1]), "i32.wast:432", 1);

// i32.wast:433
assert_return(() => call($1, "ge_u", [1, 0]), "i32.wast:433", 1);

// i32.wast:434
assert_return(() => call($1, "ge_u", [0, 1]), "i32.wast:434", 0);

// i32.wast:435
assert_return(() => call($1, "ge_u", [-2_147_483_648, 0]), "i32.wast:435", 1);

// i32.wast:436
assert_return(() => call($1, "ge_u", [0, -2_147_483_648]), "i32.wast:436", 0);

// i32.wast:437
assert_return(() => call($1, "ge_u", [-2_147_483_648, -1]), "i32.wast:437", 0);

// i32.wast:438
assert_return(() => call($1, "ge_u", [-1, -2_147_483_648]), "i32.wast:438", 1);

// i32.wast:439
assert_return(() => call($1, "ge_u", [-2_147_483_648, 2_147_483_647]), "i32.wast:439", 1);

// i32.wast:440
assert_return(() => call($1, "ge_u", [2_147_483_647, -2_147_483_648]), "i32.wast:440", 0);

// i32.wast:443
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8a\x80\x80\x80\x00\x01\x84\x80\x80\x80\x00\x00\x45\x1a\x0b", "i32.wast:443");

// i32.wast:451
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8f\x80\x80\x80\x00\x01\x89\x80\x80\x80\x00\x00\x41\x00\x02\x40\x45\x1a\x0b\x0b", "i32.wast:451");

// i32.wast:460
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8f\x80\x80\x80\x00\x01\x89\x80\x80\x80\x00\x00\x41\x00\x03\x40\x45\x1a\x0b\x0b", "i32.wast:460");

// i32.wast:469
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x00\x41\x00\x04\x40\x45\x1a\x0b\x0b", "i32.wast:469");

// i32.wast:478
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x94\x80\x80\x80\x00\x01\x8e\x80\x80\x80\x00\x00\x41\x00\x41\x00\x04\x7f\x41\x00\x05\x45\x0b\x1a\x0b", "i32.wast:478");

// i32.wast:487
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x00\x02\x40\x45\x0c\x00\x1a\x0b\x0b", "i32.wast:487");

// i32.wast:496
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x93\x80\x80\x80\x00\x01\x8d\x80\x80\x80\x00\x00\x41\x00\x02\x40\x45\x41\x01\x0d\x00\x1a\x0b\x0b", "i32.wast:496");

// i32.wast:505
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x41\x00\x02\x40\x45\x0e\x00\x00\x1a\x0b\x0b", "i32.wast:505");

// i32.wast:514
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x45\x0f\x1a\x0b", "i32.wast:514");

// i32.wast:522
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8f\x80\x80\x80\x00\x01\x89\x80\x80\x80\x00\x00\x45\x41\x01\x41\x02\x1b\x1a\x0b", "i32.wast:522");

// i32.wast:530
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x0a\x95\x80\x80\x80\x00\x02\x86\x80\x80\x80\x00\x00\x45\x10\x01\x1a\x0b\x84\x80\x80\x80\x00\x00\x20\x00\x0b", "i32.wast:530");

// i32.wast:539
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x01\x7f\x01\x7f\x60\x00\x00\x03\x83\x80\x80\x80\x00\x02\x00\x01\x04\x85\x80\x80\x80\x00\x01\x70\x01\x01\x01\x09\x89\x80\x80\x80\x00\x01\x04\x41\x00\x0b\x01\xd2\x00\x0b\x0a\x9b\x80\x80\x80\x00\x02\x84\x80\x80\x80\x00\x00\x20\x00\x0b\x8c\x80\x80\x80\x00\x00\x02\x7f\x45\x41\x00\x11\x00\x00\x1a\x0b\x0b", "i32.wast:539");

// i32.wast:555
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x01\x01\x7f\x45\x21\x00\x20\x00\x1a\x0b", "i32.wast:555");

// i32.wast:564
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x01\x01\x7f\x45\x22\x00\x1a\x0b", "i32.wast:564");

// i32.wast:573
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x06\x86\x80\x80\x80\x00\x01\x7f\x01\x41\x00\x0b\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x00\x45\x24\x00\x23\x00\x1a\x0b", "i32.wast:573");

// i32.wast:582
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x00\x0a\x8c\x80\x80\x80\x00\x01\x86\x80\x80\x80\x00\x00\x45\x40\x00\x1a\x0b", "i32.wast:582");

// i32.wast:591
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x00\x0a\x8d\x80\x80\x80\x00\x01\x87\x80\x80\x80\x00\x00\x45\x28\x02\x00\x1a\x0b", "i32.wast:591");

// i32.wast:600
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x01\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x00\x45\x41\x01\x36\x02\x00\x0b", "i32.wast:600");

// i32.wast:610
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8a\x80\x80\x80\x00\x01\x84\x80\x80\x80\x00\x00\x6a\x1a\x0b", "i32.wast:610");

// i32.wast:618
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8c\x80\x80\x80\x00\x01\x86\x80\x80\x80\x00\x00\x41\x00\x6a\x1a\x0b", "i32.wast:618");

// i32.wast:626
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x00\x41\x00\x02\x40\x6a\x1a\x0b\x0b", "i32.wast:626");

// i32.wast:635
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x00\x02\x40\x41\x00\x6a\x1a\x0b\x0b", "i32.wast:635");

// i32.wast:644
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x00\x41\x00\x03\x40\x6a\x1a\x0b\x0b", "i32.wast:644");

// i32.wast:653
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x00\x03\x40\x41\x00\x6a\x1a\x0b\x0b", "i32.wast:653");

// i32.wast:662
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x93\x80\x80\x80\x00\x01\x8d\x80\x80\x80\x00\x00\x41\x00\x41\x00\x41\x00\x6a\x04\x40\x1a\x0b\x0b", "i32.wast:662");

// i32.wast:671
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x94\x80\x80\x80\x00\x01\x8e\x80\x80\x80\x00\x00\x41\x00\x41\x00\x41\x00\x04\x40\x6a\x05\x1a\x0b\x0b", "i32.wast:671");

// i32.wast:680
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x41\x00\x41\x00\x41\x00\x04\x7f\x41\x00\x05\x6a\x41\x00\x0b\x1a\x1a\x0b", "i32.wast:680");

// i32.wast:690
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x94\x80\x80\x80\x00\x01\x8e\x80\x80\x80\x00\x00\x41\x00\x41\x00\x04\x7f\x41\x00\x05\x6a\x0b\x1a\x0b", "i32.wast:690");

// i32.wast:700
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x93\x80\x80\x80\x00\x01\x8d\x80\x80\x80\x00\x00\x41\x00\x41\x00\x02\x40\x6a\x0c\x00\x1a\x0b\x0b", "i32.wast:700");

// i32.wast:709
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x93\x80\x80\x80\x00\x01\x8d\x80\x80\x80\x00\x00\x41\x00\x02\x40\x41\x00\x6a\x0c\x00\x1a\x0b\x0b", "i32.wast:709");

// i32.wast:718
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x41\x00\x41\x00\x02\x40\x6a\x41\x01\x0d\x00\x1a\x0b\x0b", "i32.wast:718");

// i32.wast:727
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x41\x00\x02\x40\x41\x00\x6a\x41\x01\x0d\x00\x1a\x0b\x0b", "i32.wast:727");

// i32.wast:736
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x94\x80\x80\x80\x00\x01\x8e\x80\x80\x80\x00\x00\x41\x00\x41\x00\x02\x40\x6a\x0e\x00\x00\x1a\x0b\x0b", "i32.wast:736");

// i32.wast:745
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x94\x80\x80\x80\x00\x01\x8e\x80\x80\x80\x00\x00\x41\x00\x02\x40\x41\x00\x6a\x0e\x00\x00\x1a\x0b\x0b", "i32.wast:745");

// i32.wast:754
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x6a\x0f\x1a\x0b", "i32.wast:754");

// i32.wast:762
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8d\x80\x80\x80\x00\x01\x87\x80\x80\x80\x00\x00\x41\x00\x6a\x0f\x1a\x0b", "i32.wast:762");

// i32.wast:770
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8f\x80\x80\x80\x00\x01\x89\x80\x80\x80\x00\x00\x6a\x41\x01\x41\x02\x1b\x1a\x0b", "i32.wast:770");

// i32.wast:778
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x00\x6a\x41\x01\x41\x02\x1b\x1a\x0b", "i32.wast:778");

// i32.wast:786
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8a\x80\x80\x80\x00\x02\x60\x00\x00\x60\x02\x7f\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x0a\x95\x80\x80\x80\x00\x02\x86\x80\x80\x80\x00\x00\x6a\x10\x01\x1a\x0b\x84\x80\x80\x80\x00\x00\x20\x00\x0b", "i32.wast:786");

// i32.wast:795
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8a\x80\x80\x80\x00\x02\x60\x00\x00\x60\x02\x7f\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x0a\x97\x80\x80\x80\x00\x02\x88\x80\x80\x80\x00\x00\x41\x00\x6a\x10\x01\x1a\x0b\x84\x80\x80\x80\x00\x00\x20\x00\x0b", "i32.wast:795");

// i32.wast:804
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x01\x7f\x01\x7f\x60\x00\x00\x03\x83\x80\x80\x80\x00\x02\x00\x01\x04\x85\x80\x80\x80\x00\x01\x70\x01\x01\x01\x09\x89\x80\x80\x80\x00\x01\x04\x41\x00\x0b\x01\xd2\x00\x0b\x0a\x9b\x80\x80\x80\x00\x02\x84\x80\x80\x80\x00\x00\x20\x00\x0b\x8c\x80\x80\x80\x00\x00\x02\x7f\x6a\x41\x00\x11\x00\x00\x1a\x0b\x0b", "i32.wast:804");

// i32.wast:820
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x01\x7f\x01\x7f\x60\x00\x00\x03\x83\x80\x80\x80\x00\x02\x00\x01\x04\x85\x80\x80\x80\x00\x01\x70\x01\x01\x01\x09\x89\x80\x80\x80\x00\x01\x04\x41\x00\x0b\x01\xd2\x00\x0b\x0a\x9d\x80\x80\x80\x00\x02\x84\x80\x80\x80\x00\x00\x20\x00\x0b\x8e\x80\x80\x80\x00\x00\x02\x7f\x41\x00\x6a\x41\x00\x11\x00\x00\x1a\x0b\x0b", "i32.wast:820");

// i32.wast:836
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x01\x01\x7f\x6a\x21\x00\x20\x00\x1a\x0b", "i32.wast:836");

// i32.wast:845
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x01\x01\x7f\x41\x00\x6a\x21\x00\x20\x00\x1a\x0b", "i32.wast:845");

// i32.wast:854
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x01\x01\x7f\x6a\x22\x00\x1a\x0b", "i32.wast:854");

// i32.wast:863
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x01\x01\x7f\x41\x00\x6a\x22\x00\x1a\x0b", "i32.wast:863");

// i32.wast:872
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x06\x86\x80\x80\x80\x00\x01\x7f\x01\x41\x00\x0b\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x00\x6a\x24\x00\x23\x00\x1a\x0b", "i32.wast:872");

// i32.wast:881
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x06\x86\x80\x80\x80\x00\x01\x7f\x01\x41\x00\x0b\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x00\x6a\x24\x00\x23\x00\x1a\x0b", "i32.wast:881");

// i32.wast:890
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x00\x0a\x8c\x80\x80\x80\x00\x01\x86\x80\x80\x80\x00\x00\x6a\x40\x00\x1a\x0b", "i32.wast:890");

// i32.wast:899
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x00\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x00\x41\x00\x6a\x40\x00\x1a\x0b", "i32.wast:899");

// i32.wast:908
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x00\x0a\x8d\x80\x80\x80\x00\x01\x87\x80\x80\x80\x00\x00\x6a\x28\x02\x00\x1a\x0b", "i32.wast:908");

// i32.wast:917
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x00\x0a\x8f\x80\x80\x80\x00\x01\x89\x80\x80\x80\x00\x00\x41\x00\x6a\x28\x02\x00\x1a\x0b", "i32.wast:917");

// i32.wast:926
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x01\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x00\x6a\x41\x01\x36\x02\x00\x0b", "i32.wast:926");

// i32.wast:935
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x01\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x41\x01\x6a\x41\x00\x36\x02\x00\x0b", "i32.wast:935");

// i32.wast:948
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x6a\x0b", "i32.wast:948");

// i32.wast:949
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x71\x0b", "i32.wast:949");

// i32.wast:950
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x6d\x0b", "i32.wast:950");

// i32.wast:951
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x6e\x0b", "i32.wast:951");

// i32.wast:952
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x6c\x0b", "i32.wast:952");

// i32.wast:953
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x72\x0b", "i32.wast:953");

// i32.wast:954
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x6f\x0b", "i32.wast:954");

// i32.wast:955
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x70\x0b", "i32.wast:955");

// i32.wast:956
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x77\x0b", "i32.wast:956");

// i32.wast:957
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x78\x0b", "i32.wast:957");

// i32.wast:958
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x74\x0b", "i32.wast:958");

// i32.wast:959
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x75\x0b", "i32.wast:959");

// i32.wast:960
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x76\x0b", "i32.wast:960");

// i32.wast:961
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x6b\x0b", "i32.wast:961");

// i32.wast:962
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x73\x0b", "i32.wast:962");

// i32.wast:963
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x42\x00\x45\x0b", "i32.wast:963");

// i32.wast:964
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x42\x00\x67\x0b", "i32.wast:964");

// i32.wast:965
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x42\x00\x68\x0b", "i32.wast:965");

// i32.wast:966
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\x42\x00\x69\x0b", "i32.wast:966");

// i32.wast:967
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x46\x0b", "i32.wast:967");

// i32.wast:968
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x4e\x0b", "i32.wast:968");

// i32.wast:969
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x4f\x0b", "i32.wast:969");

// i32.wast:970
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x4a\x0b", "i32.wast:970");

// i32.wast:971
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x4b\x0b", "i32.wast:971");

// i32.wast:972
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x4c\x0b", "i32.wast:972");

// i32.wast:973
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x4d\x0b", "i32.wast:973");

// i32.wast:974
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x48\x0b", "i32.wast:974");

// i32.wast:975
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x49\x0b", "i32.wast:975");

// i32.wast:976
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x42\x00\x43\x00\x00\x00\x00\x47\x0b", "i32.wast:976");

// i32.wast:978
assert_malformed("\x3c\x6d\x61\x6c\x66\x6f\x72\x6d\x65\x64\x20\x71\x75\x6f\x74\x65\x3e", "i32.wast:978");

// i32.wast:982
assert_malformed("\x3c\x6d\x61\x6c\x66\x6f\x72\x6d\x65\x64\x20\x71\x75\x6f\x74\x65\x3e", "i32.wast:982");
reinitializeRegistry();
})();
