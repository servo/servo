(function table_copy_wast_js() {

// table_copy.wast:6
let $1 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x86\x80\x80\x80\x00\x05\x00\x00\x00\x00\x00\x07\x9f\x80\x80\x80\x00\x05\x03\x65\x66\x30\x00\x00\x03\x65\x66\x31\x00\x01\x03\x65\x66\x32\x00\x02\x03\x65\x66\x33\x00\x03\x03\x65\x66\x34\x00\x04\x0a\xae\x80\x80\x80\x00\x05\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b");

// table_copy.wast:13
register("a", $1)

// table_copy.wast:15
let $2 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x02\x01\x41\x03\x0b\x00\x04\x01\x03\x01\x04\x02\x01\x41\x0b\x0b\x00\x05\x06\x03\x02\x05\x07\x0a\xce\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x83\x80\x80\x80\x00\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b");

// table_copy.wast:45
run(() => call($2, "test", []));

// table_copy.wast:46
assert_trap(() => call($2, "check_t0", [0]));

// table_copy.wast:47
assert_trap(() => call($2, "check_t0", [1]));

// table_copy.wast:48
assert_return(() => call($2, "check_t0", [2]), 3);

// table_copy.wast:49
assert_return(() => call($2, "check_t0", [3]), 1);

// table_copy.wast:50
assert_return(() => call($2, "check_t0", [4]), 4);

// table_copy.wast:51
assert_return(() => call($2, "check_t0", [5]), 1);

// table_copy.wast:52
assert_trap(() => call($2, "check_t0", [6]));

// table_copy.wast:53
assert_trap(() => call($2, "check_t0", [7]));

// table_copy.wast:54
assert_trap(() => call($2, "check_t0", [8]));

// table_copy.wast:55
assert_trap(() => call($2, "check_t0", [9]));

// table_copy.wast:56
assert_trap(() => call($2, "check_t0", [10]));

// table_copy.wast:57
assert_trap(() => call($2, "check_t0", [11]));

// table_copy.wast:58
assert_return(() => call($2, "check_t0", [12]), 7);

// table_copy.wast:59
assert_return(() => call($2, "check_t0", [13]), 5);

// table_copy.wast:60
assert_return(() => call($2, "check_t0", [14]), 2);

// table_copy.wast:61
assert_return(() => call($2, "check_t0", [15]), 3);

// table_copy.wast:62
assert_return(() => call($2, "check_t0", [16]), 6);

// table_copy.wast:63
assert_trap(() => call($2, "check_t0", [17]));

// table_copy.wast:64
assert_trap(() => call($2, "check_t0", [18]));

// table_copy.wast:65
assert_trap(() => call($2, "check_t0", [19]));

// table_copy.wast:66
assert_trap(() => call($2, "check_t0", [20]));

// table_copy.wast:67
assert_trap(() => call($2, "check_t0", [21]));

// table_copy.wast:68
assert_trap(() => call($2, "check_t0", [22]));

// table_copy.wast:69
assert_trap(() => call($2, "check_t0", [23]));

// table_copy.wast:70
assert_trap(() => call($2, "check_t0", [24]));

// table_copy.wast:71
assert_trap(() => call($2, "check_t0", [25]));

// table_copy.wast:72
assert_trap(() => call($2, "check_t0", [26]));

// table_copy.wast:73
assert_trap(() => call($2, "check_t0", [27]));

// table_copy.wast:74
assert_trap(() => call($2, "check_t0", [28]));

// table_copy.wast:75
assert_trap(() => call($2, "check_t0", [29]));

// table_copy.wast:76
assert_trap(() => call($2, "check_t1", [0]));

// table_copy.wast:77
assert_trap(() => call($2, "check_t1", [1]));

// table_copy.wast:78
assert_trap(() => call($2, "check_t1", [2]));

// table_copy.wast:79
assert_return(() => call($2, "check_t1", [3]), 1);

// table_copy.wast:80
assert_return(() => call($2, "check_t1", [4]), 3);

// table_copy.wast:81
assert_return(() => call($2, "check_t1", [5]), 1);

// table_copy.wast:82
assert_return(() => call($2, "check_t1", [6]), 4);

// table_copy.wast:83
assert_trap(() => call($2, "check_t1", [7]));

// table_copy.wast:84
assert_trap(() => call($2, "check_t1", [8]));

// table_copy.wast:85
assert_trap(() => call($2, "check_t1", [9]));

// table_copy.wast:86
assert_trap(() => call($2, "check_t1", [10]));

// table_copy.wast:87
assert_return(() => call($2, "check_t1", [11]), 6);

// table_copy.wast:88
assert_return(() => call($2, "check_t1", [12]), 3);

// table_copy.wast:89
assert_return(() => call($2, "check_t1", [13]), 2);

// table_copy.wast:90
assert_return(() => call($2, "check_t1", [14]), 5);

// table_copy.wast:91
assert_return(() => call($2, "check_t1", [15]), 7);

// table_copy.wast:92
assert_trap(() => call($2, "check_t1", [16]));

// table_copy.wast:93
assert_trap(() => call($2, "check_t1", [17]));

// table_copy.wast:94
assert_trap(() => call($2, "check_t1", [18]));

// table_copy.wast:95
assert_trap(() => call($2, "check_t1", [19]));

// table_copy.wast:96
assert_trap(() => call($2, "check_t1", [20]));

// table_copy.wast:97
assert_trap(() => call($2, "check_t1", [21]));

// table_copy.wast:98
assert_trap(() => call($2, "check_t1", [22]));

// table_copy.wast:99
assert_trap(() => call($2, "check_t1", [23]));

// table_copy.wast:100
assert_trap(() => call($2, "check_t1", [24]));

// table_copy.wast:101
assert_trap(() => call($2, "check_t1", [25]));

// table_copy.wast:102
assert_trap(() => call($2, "check_t1", [26]));

// table_copy.wast:103
assert_trap(() => call($2, "check_t1", [27]));

// table_copy.wast:104
assert_trap(() => call($2, "check_t1", [28]));

// table_copy.wast:105
assert_trap(() => call($2, "check_t1", [29]));

// table_copy.wast:107
let $3 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x02\x01\x41\x03\x0b\x00\x04\x01\x03\x01\x04\x02\x01\x41\x0b\x0b\x00\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0d\x41\x02\x41\x03\xfc\x0e\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b");

// table_copy.wast:137
run(() => call($3, "test", []));

// table_copy.wast:138
assert_trap(() => call($3, "check_t0", [0]));

// table_copy.wast:139
assert_trap(() => call($3, "check_t0", [1]));

// table_copy.wast:140
assert_return(() => call($3, "check_t0", [2]), 3);

// table_copy.wast:141
assert_return(() => call($3, "check_t0", [3]), 1);

// table_copy.wast:142
assert_return(() => call($3, "check_t0", [4]), 4);

// table_copy.wast:143
assert_return(() => call($3, "check_t0", [5]), 1);

// table_copy.wast:144
assert_trap(() => call($3, "check_t0", [6]));

// table_copy.wast:145
assert_trap(() => call($3, "check_t0", [7]));

// table_copy.wast:146
assert_trap(() => call($3, "check_t0", [8]));

// table_copy.wast:147
assert_trap(() => call($3, "check_t0", [9]));

// table_copy.wast:148
assert_trap(() => call($3, "check_t0", [10]));

// table_copy.wast:149
assert_trap(() => call($3, "check_t0", [11]));

// table_copy.wast:150
assert_return(() => call($3, "check_t0", [12]), 7);

// table_copy.wast:151
assert_return(() => call($3, "check_t0", [13]), 3);

// table_copy.wast:152
assert_return(() => call($3, "check_t0", [14]), 1);

// table_copy.wast:153
assert_return(() => call($3, "check_t0", [15]), 4);

// table_copy.wast:154
assert_return(() => call($3, "check_t0", [16]), 6);

// table_copy.wast:155
assert_trap(() => call($3, "check_t0", [17]));

// table_copy.wast:156
assert_trap(() => call($3, "check_t0", [18]));

// table_copy.wast:157
assert_trap(() => call($3, "check_t0", [19]));

// table_copy.wast:158
assert_trap(() => call($3, "check_t0", [20]));

// table_copy.wast:159
assert_trap(() => call($3, "check_t0", [21]));

// table_copy.wast:160
assert_trap(() => call($3, "check_t0", [22]));

// table_copy.wast:161
assert_trap(() => call($3, "check_t0", [23]));

// table_copy.wast:162
assert_trap(() => call($3, "check_t0", [24]));

// table_copy.wast:163
assert_trap(() => call($3, "check_t0", [25]));

// table_copy.wast:164
assert_trap(() => call($3, "check_t0", [26]));

// table_copy.wast:165
assert_trap(() => call($3, "check_t0", [27]));

// table_copy.wast:166
assert_trap(() => call($3, "check_t0", [28]));

// table_copy.wast:167
assert_trap(() => call($3, "check_t0", [29]));

// table_copy.wast:168
assert_trap(() => call($3, "check_t1", [0]));

// table_copy.wast:169
assert_trap(() => call($3, "check_t1", [1]));

// table_copy.wast:170
assert_trap(() => call($3, "check_t1", [2]));

// table_copy.wast:171
assert_return(() => call($3, "check_t1", [3]), 1);

// table_copy.wast:172
assert_return(() => call($3, "check_t1", [4]), 3);

// table_copy.wast:173
assert_return(() => call($3, "check_t1", [5]), 1);

// table_copy.wast:174
assert_return(() => call($3, "check_t1", [6]), 4);

// table_copy.wast:175
assert_trap(() => call($3, "check_t1", [7]));

// table_copy.wast:176
assert_trap(() => call($3, "check_t1", [8]));

// table_copy.wast:177
assert_trap(() => call($3, "check_t1", [9]));

// table_copy.wast:178
assert_trap(() => call($3, "check_t1", [10]));

// table_copy.wast:179
assert_return(() => call($3, "check_t1", [11]), 6);

// table_copy.wast:180
assert_return(() => call($3, "check_t1", [12]), 3);

// table_copy.wast:181
assert_return(() => call($3, "check_t1", [13]), 2);

// table_copy.wast:182
assert_return(() => call($3, "check_t1", [14]), 5);

// table_copy.wast:183
assert_return(() => call($3, "check_t1", [15]), 7);

// table_copy.wast:184
assert_trap(() => call($3, "check_t1", [16]));

// table_copy.wast:185
assert_trap(() => call($3, "check_t1", [17]));

// table_copy.wast:186
assert_trap(() => call($3, "check_t1", [18]));

// table_copy.wast:187
assert_trap(() => call($3, "check_t1", [19]));

// table_copy.wast:188
assert_trap(() => call($3, "check_t1", [20]));

// table_copy.wast:189
assert_trap(() => call($3, "check_t1", [21]));

// table_copy.wast:190
assert_trap(() => call($3, "check_t1", [22]));

// table_copy.wast:191
assert_trap(() => call($3, "check_t1", [23]));

// table_copy.wast:192
assert_trap(() => call($3, "check_t1", [24]));

// table_copy.wast:193
assert_trap(() => call($3, "check_t1", [25]));

// table_copy.wast:194
assert_trap(() => call($3, "check_t1", [26]));

// table_copy.wast:195
assert_trap(() => call($3, "check_t1", [27]));

// table_copy.wast:196
assert_trap(() => call($3, "check_t1", [28]));

// table_copy.wast:197
assert_trap(() => call($3, "check_t1", [29]));

// table_copy.wast:199
let $4 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x02\x01\x41\x03\x0b\x00\x04\x01\x03\x01\x04\x02\x01\x41\x0b\x0b\x00\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x19\x41\x0f\x41\x02\xfc\x0e\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b");

// table_copy.wast:229
run(() => call($4, "test", []));

// table_copy.wast:230
assert_trap(() => call($4, "check_t0", [0]));

// table_copy.wast:231
assert_trap(() => call($4, "check_t0", [1]));

// table_copy.wast:232
assert_return(() => call($4, "check_t0", [2]), 3);

// table_copy.wast:233
assert_return(() => call($4, "check_t0", [3]), 1);

// table_copy.wast:234
assert_return(() => call($4, "check_t0", [4]), 4);

// table_copy.wast:235
assert_return(() => call($4, "check_t0", [5]), 1);

// table_copy.wast:236
assert_trap(() => call($4, "check_t0", [6]));

// table_copy.wast:237
assert_trap(() => call($4, "check_t0", [7]));

// table_copy.wast:238
assert_trap(() => call($4, "check_t0", [8]));

// table_copy.wast:239
assert_trap(() => call($4, "check_t0", [9]));

// table_copy.wast:240
assert_trap(() => call($4, "check_t0", [10]));

// table_copy.wast:241
assert_trap(() => call($4, "check_t0", [11]));

// table_copy.wast:242
assert_return(() => call($4, "check_t0", [12]), 7);

// table_copy.wast:243
assert_return(() => call($4, "check_t0", [13]), 5);

// table_copy.wast:244
assert_return(() => call($4, "check_t0", [14]), 2);

// table_copy.wast:245
assert_return(() => call($4, "check_t0", [15]), 3);

// table_copy.wast:246
assert_return(() => call($4, "check_t0", [16]), 6);

// table_copy.wast:247
assert_trap(() => call($4, "check_t0", [17]));

// table_copy.wast:248
assert_trap(() => call($4, "check_t0", [18]));

// table_copy.wast:249
assert_trap(() => call($4, "check_t0", [19]));

// table_copy.wast:250
assert_trap(() => call($4, "check_t0", [20]));

// table_copy.wast:251
assert_trap(() => call($4, "check_t0", [21]));

// table_copy.wast:252
assert_trap(() => call($4, "check_t0", [22]));

// table_copy.wast:253
assert_trap(() => call($4, "check_t0", [23]));

// table_copy.wast:254
assert_trap(() => call($4, "check_t0", [24]));

// table_copy.wast:255
assert_return(() => call($4, "check_t0", [25]), 3);

// table_copy.wast:256
assert_return(() => call($4, "check_t0", [26]), 6);

// table_copy.wast:257
assert_trap(() => call($4, "check_t0", [27]));

// table_copy.wast:258
assert_trap(() => call($4, "check_t0", [28]));

// table_copy.wast:259
assert_trap(() => call($4, "check_t0", [29]));

// table_copy.wast:260
assert_trap(() => call($4, "check_t1", [0]));

// table_copy.wast:261
assert_trap(() => call($4, "check_t1", [1]));

// table_copy.wast:262
assert_trap(() => call($4, "check_t1", [2]));

// table_copy.wast:263
assert_return(() => call($4, "check_t1", [3]), 1);

// table_copy.wast:264
assert_return(() => call($4, "check_t1", [4]), 3);

// table_copy.wast:265
assert_return(() => call($4, "check_t1", [5]), 1);

// table_copy.wast:266
assert_return(() => call($4, "check_t1", [6]), 4);

// table_copy.wast:267
assert_trap(() => call($4, "check_t1", [7]));

// table_copy.wast:268
assert_trap(() => call($4, "check_t1", [8]));

// table_copy.wast:269
assert_trap(() => call($4, "check_t1", [9]));

// table_copy.wast:270
assert_trap(() => call($4, "check_t1", [10]));

// table_copy.wast:271
assert_return(() => call($4, "check_t1", [11]), 6);

// table_copy.wast:272
assert_return(() => call($4, "check_t1", [12]), 3);

// table_copy.wast:273
assert_return(() => call($4, "check_t1", [13]), 2);

// table_copy.wast:274
assert_return(() => call($4, "check_t1", [14]), 5);

// table_copy.wast:275
assert_return(() => call($4, "check_t1", [15]), 7);

// table_copy.wast:276
assert_trap(() => call($4, "check_t1", [16]));

// table_copy.wast:277
assert_trap(() => call($4, "check_t1", [17]));

// table_copy.wast:278
assert_trap(() => call($4, "check_t1", [18]));

// table_copy.wast:279
assert_trap(() => call($4, "check_t1", [19]));

// table_copy.wast:280
assert_trap(() => call($4, "check_t1", [20]));

// table_copy.wast:281
assert_trap(() => call($4, "check_t1", [21]));

// table_copy.wast:282
assert_trap(() => call($4, "check_t1", [22]));

// table_copy.wast:283
assert_trap(() => call($4, "check_t1", [23]));

// table_copy.wast:284
assert_trap(() => call($4, "check_t1", [24]));

// table_copy.wast:285
assert_trap(() => call($4, "check_t1", [25]));

// table_copy.wast:286
assert_trap(() => call($4, "check_t1", [26]));

// table_copy.wast:287
assert_trap(() => call($4, "check_t1", [27]));

// table_copy.wast:288
assert_trap(() => call($4, "check_t1", [28]));

// table_copy.wast:289
assert_trap(() => call($4, "check_t1", [29]));

// table_copy.wast:291
let $5 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x02\x01\x41\x03\x0b\x00\x04\x01\x03\x01\x04\x02\x01\x41\x0b\x0b\x00\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0d\x41\x19\x41\x03\xfc\x0e\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b");

// table_copy.wast:321
run(() => call($5, "test", []));

// table_copy.wast:322
assert_trap(() => call($5, "check_t0", [0]));

// table_copy.wast:323
assert_trap(() => call($5, "check_t0", [1]));

// table_copy.wast:324
assert_return(() => call($5, "check_t0", [2]), 3);

// table_copy.wast:325
assert_return(() => call($5, "check_t0", [3]), 1);

// table_copy.wast:326
assert_return(() => call($5, "check_t0", [4]), 4);

// table_copy.wast:327
assert_return(() => call($5, "check_t0", [5]), 1);

// table_copy.wast:328
assert_trap(() => call($5, "check_t0", [6]));

// table_copy.wast:329
assert_trap(() => call($5, "check_t0", [7]));

// table_copy.wast:330
assert_trap(() => call($5, "check_t0", [8]));

// table_copy.wast:331
assert_trap(() => call($5, "check_t0", [9]));

// table_copy.wast:332
assert_trap(() => call($5, "check_t0", [10]));

// table_copy.wast:333
assert_trap(() => call($5, "check_t0", [11]));

// table_copy.wast:334
assert_return(() => call($5, "check_t0", [12]), 7);

// table_copy.wast:335
assert_trap(() => call($5, "check_t0", [13]));

// table_copy.wast:336
assert_trap(() => call($5, "check_t0", [14]));

// table_copy.wast:337
assert_trap(() => call($5, "check_t0", [15]));

// table_copy.wast:338
assert_return(() => call($5, "check_t0", [16]), 6);

// table_copy.wast:339
assert_trap(() => call($5, "check_t0", [17]));

// table_copy.wast:340
assert_trap(() => call($5, "check_t0", [18]));

// table_copy.wast:341
assert_trap(() => call($5, "check_t0", [19]));

// table_copy.wast:342
assert_trap(() => call($5, "check_t0", [20]));

// table_copy.wast:343
assert_trap(() => call($5, "check_t0", [21]));

// table_copy.wast:344
assert_trap(() => call($5, "check_t0", [22]));

// table_copy.wast:345
assert_trap(() => call($5, "check_t0", [23]));

// table_copy.wast:346
assert_trap(() => call($5, "check_t0", [24]));

// table_copy.wast:347
assert_trap(() => call($5, "check_t0", [25]));

// table_copy.wast:348
assert_trap(() => call($5, "check_t0", [26]));

// table_copy.wast:349
assert_trap(() => call($5, "check_t0", [27]));

// table_copy.wast:350
assert_trap(() => call($5, "check_t0", [28]));

// table_copy.wast:351
assert_trap(() => call($5, "check_t0", [29]));

// table_copy.wast:352
assert_trap(() => call($5, "check_t1", [0]));

// table_copy.wast:353
assert_trap(() => call($5, "check_t1", [1]));

// table_copy.wast:354
assert_trap(() => call($5, "check_t1", [2]));

// table_copy.wast:355
assert_return(() => call($5, "check_t1", [3]), 1);

// table_copy.wast:356
assert_return(() => call($5, "check_t1", [4]), 3);

// table_copy.wast:357
assert_return(() => call($5, "check_t1", [5]), 1);

// table_copy.wast:358
assert_return(() => call($5, "check_t1", [6]), 4);

// table_copy.wast:359
assert_trap(() => call($5, "check_t1", [7]));

// table_copy.wast:360
assert_trap(() => call($5, "check_t1", [8]));

// table_copy.wast:361
assert_trap(() => call($5, "check_t1", [9]));

// table_copy.wast:362
assert_trap(() => call($5, "check_t1", [10]));

// table_copy.wast:363
assert_return(() => call($5, "check_t1", [11]), 6);

// table_copy.wast:364
assert_return(() => call($5, "check_t1", [12]), 3);

// table_copy.wast:365
assert_return(() => call($5, "check_t1", [13]), 2);

// table_copy.wast:366
assert_return(() => call($5, "check_t1", [14]), 5);

// table_copy.wast:367
assert_return(() => call($5, "check_t1", [15]), 7);

// table_copy.wast:368
assert_trap(() => call($5, "check_t1", [16]));

// table_copy.wast:369
assert_trap(() => call($5, "check_t1", [17]));

// table_copy.wast:370
assert_trap(() => call($5, "check_t1", [18]));

// table_copy.wast:371
assert_trap(() => call($5, "check_t1", [19]));

// table_copy.wast:372
assert_trap(() => call($5, "check_t1", [20]));

// table_copy.wast:373
assert_trap(() => call($5, "check_t1", [21]));

// table_copy.wast:374
assert_trap(() => call($5, "check_t1", [22]));

// table_copy.wast:375
assert_trap(() => call($5, "check_t1", [23]));

// table_copy.wast:376
assert_trap(() => call($5, "check_t1", [24]));

// table_copy.wast:377
assert_trap(() => call($5, "check_t1", [25]));

// table_copy.wast:378
assert_trap(() => call($5, "check_t1", [26]));

// table_copy.wast:379
assert_trap(() => call($5, "check_t1", [27]));

// table_copy.wast:380
assert_trap(() => call($5, "check_t1", [28]));

// table_copy.wast:381
assert_trap(() => call($5, "check_t1", [29]));

// table_copy.wast:383
let $6 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x02\x01\x41\x03\x0b\x00\x04\x01\x03\x01\x04\x02\x01\x41\x0b\x0b\x00\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x14\x41\x16\x41\x04\xfc\x0e\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b");

// table_copy.wast:413
run(() => call($6, "test", []));

// table_copy.wast:414
assert_trap(() => call($6, "check_t0", [0]));

// table_copy.wast:415
assert_trap(() => call($6, "check_t0", [1]));

// table_copy.wast:416
assert_return(() => call($6, "check_t0", [2]), 3);

// table_copy.wast:417
assert_return(() => call($6, "check_t0", [3]), 1);

// table_copy.wast:418
assert_return(() => call($6, "check_t0", [4]), 4);

// table_copy.wast:419
assert_return(() => call($6, "check_t0", [5]), 1);

// table_copy.wast:420
assert_trap(() => call($6, "check_t0", [6]));

// table_copy.wast:421
assert_trap(() => call($6, "check_t0", [7]));

// table_copy.wast:422
assert_trap(() => call($6, "check_t0", [8]));

// table_copy.wast:423
assert_trap(() => call($6, "check_t0", [9]));

// table_copy.wast:424
assert_trap(() => call($6, "check_t0", [10]));

// table_copy.wast:425
assert_trap(() => call($6, "check_t0", [11]));

// table_copy.wast:426
assert_return(() => call($6, "check_t0", [12]), 7);

// table_copy.wast:427
assert_return(() => call($6, "check_t0", [13]), 5);

// table_copy.wast:428
assert_return(() => call($6, "check_t0", [14]), 2);

// table_copy.wast:429
assert_return(() => call($6, "check_t0", [15]), 3);

// table_copy.wast:430
assert_return(() => call($6, "check_t0", [16]), 6);

// table_copy.wast:431
assert_trap(() => call($6, "check_t0", [17]));

// table_copy.wast:432
assert_trap(() => call($6, "check_t0", [18]));

// table_copy.wast:433
assert_trap(() => call($6, "check_t0", [19]));

// table_copy.wast:434
assert_trap(() => call($6, "check_t0", [20]));

// table_copy.wast:435
assert_trap(() => call($6, "check_t0", [21]));

// table_copy.wast:436
assert_trap(() => call($6, "check_t0", [22]));

// table_copy.wast:437
assert_trap(() => call($6, "check_t0", [23]));

// table_copy.wast:438
assert_trap(() => call($6, "check_t0", [24]));

// table_copy.wast:439
assert_trap(() => call($6, "check_t0", [25]));

// table_copy.wast:440
assert_trap(() => call($6, "check_t0", [26]));

// table_copy.wast:441
assert_trap(() => call($6, "check_t0", [27]));

// table_copy.wast:442
assert_trap(() => call($6, "check_t0", [28]));

// table_copy.wast:443
assert_trap(() => call($6, "check_t0", [29]));

// table_copy.wast:444
assert_trap(() => call($6, "check_t1", [0]));

// table_copy.wast:445
assert_trap(() => call($6, "check_t1", [1]));

// table_copy.wast:446
assert_trap(() => call($6, "check_t1", [2]));

// table_copy.wast:447
assert_return(() => call($6, "check_t1", [3]), 1);

// table_copy.wast:448
assert_return(() => call($6, "check_t1", [4]), 3);

// table_copy.wast:449
assert_return(() => call($6, "check_t1", [5]), 1);

// table_copy.wast:450
assert_return(() => call($6, "check_t1", [6]), 4);

// table_copy.wast:451
assert_trap(() => call($6, "check_t1", [7]));

// table_copy.wast:452
assert_trap(() => call($6, "check_t1", [8]));

// table_copy.wast:453
assert_trap(() => call($6, "check_t1", [9]));

// table_copy.wast:454
assert_trap(() => call($6, "check_t1", [10]));

// table_copy.wast:455
assert_return(() => call($6, "check_t1", [11]), 6);

// table_copy.wast:456
assert_return(() => call($6, "check_t1", [12]), 3);

// table_copy.wast:457
assert_return(() => call($6, "check_t1", [13]), 2);

// table_copy.wast:458
assert_return(() => call($6, "check_t1", [14]), 5);

// table_copy.wast:459
assert_return(() => call($6, "check_t1", [15]), 7);

// table_copy.wast:460
assert_trap(() => call($6, "check_t1", [16]));

// table_copy.wast:461
assert_trap(() => call($6, "check_t1", [17]));

// table_copy.wast:462
assert_trap(() => call($6, "check_t1", [18]));

// table_copy.wast:463
assert_trap(() => call($6, "check_t1", [19]));

// table_copy.wast:464
assert_trap(() => call($6, "check_t1", [20]));

// table_copy.wast:465
assert_trap(() => call($6, "check_t1", [21]));

// table_copy.wast:466
assert_trap(() => call($6, "check_t1", [22]));

// table_copy.wast:467
assert_trap(() => call($6, "check_t1", [23]));

// table_copy.wast:468
assert_trap(() => call($6, "check_t1", [24]));

// table_copy.wast:469
assert_trap(() => call($6, "check_t1", [25]));

// table_copy.wast:470
assert_trap(() => call($6, "check_t1", [26]));

// table_copy.wast:471
assert_trap(() => call($6, "check_t1", [27]));

// table_copy.wast:472
assert_trap(() => call($6, "check_t1", [28]));

// table_copy.wast:473
assert_trap(() => call($6, "check_t1", [29]));

// table_copy.wast:475
let $7 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x02\x01\x41\x03\x0b\x00\x04\x01\x03\x01\x04\x02\x01\x41\x0b\x0b\x00\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x19\x41\x01\x41\x03\xfc\x0e\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b");

// table_copy.wast:505
run(() => call($7, "test", []));

// table_copy.wast:506
assert_trap(() => call($7, "check_t0", [0]));

// table_copy.wast:507
assert_trap(() => call($7, "check_t0", [1]));

// table_copy.wast:508
assert_return(() => call($7, "check_t0", [2]), 3);

// table_copy.wast:509
assert_return(() => call($7, "check_t0", [3]), 1);

// table_copy.wast:510
assert_return(() => call($7, "check_t0", [4]), 4);

// table_copy.wast:511
assert_return(() => call($7, "check_t0", [5]), 1);

// table_copy.wast:512
assert_trap(() => call($7, "check_t0", [6]));

// table_copy.wast:513
assert_trap(() => call($7, "check_t0", [7]));

// table_copy.wast:514
assert_trap(() => call($7, "check_t0", [8]));

// table_copy.wast:515
assert_trap(() => call($7, "check_t0", [9]));

// table_copy.wast:516
assert_trap(() => call($7, "check_t0", [10]));

// table_copy.wast:517
assert_trap(() => call($7, "check_t0", [11]));

// table_copy.wast:518
assert_return(() => call($7, "check_t0", [12]), 7);

// table_copy.wast:519
assert_return(() => call($7, "check_t0", [13]), 5);

// table_copy.wast:520
assert_return(() => call($7, "check_t0", [14]), 2);

// table_copy.wast:521
assert_return(() => call($7, "check_t0", [15]), 3);

// table_copy.wast:522
assert_return(() => call($7, "check_t0", [16]), 6);

// table_copy.wast:523
assert_trap(() => call($7, "check_t0", [17]));

// table_copy.wast:524
assert_trap(() => call($7, "check_t0", [18]));

// table_copy.wast:525
assert_trap(() => call($7, "check_t0", [19]));

// table_copy.wast:526
assert_trap(() => call($7, "check_t0", [20]));

// table_copy.wast:527
assert_trap(() => call($7, "check_t0", [21]));

// table_copy.wast:528
assert_trap(() => call($7, "check_t0", [22]));

// table_copy.wast:529
assert_trap(() => call($7, "check_t0", [23]));

// table_copy.wast:530
assert_trap(() => call($7, "check_t0", [24]));

// table_copy.wast:531
assert_trap(() => call($7, "check_t0", [25]));

// table_copy.wast:532
assert_return(() => call($7, "check_t0", [26]), 3);

// table_copy.wast:533
assert_return(() => call($7, "check_t0", [27]), 1);

// table_copy.wast:534
assert_trap(() => call($7, "check_t0", [28]));

// table_copy.wast:535
assert_trap(() => call($7, "check_t0", [29]));

// table_copy.wast:536
assert_trap(() => call($7, "check_t1", [0]));

// table_copy.wast:537
assert_trap(() => call($7, "check_t1", [1]));

// table_copy.wast:538
assert_trap(() => call($7, "check_t1", [2]));

// table_copy.wast:539
assert_return(() => call($7, "check_t1", [3]), 1);

// table_copy.wast:540
assert_return(() => call($7, "check_t1", [4]), 3);

// table_copy.wast:541
assert_return(() => call($7, "check_t1", [5]), 1);

// table_copy.wast:542
assert_return(() => call($7, "check_t1", [6]), 4);

// table_copy.wast:543
assert_trap(() => call($7, "check_t1", [7]));

// table_copy.wast:544
assert_trap(() => call($7, "check_t1", [8]));

// table_copy.wast:545
assert_trap(() => call($7, "check_t1", [9]));

// table_copy.wast:546
assert_trap(() => call($7, "check_t1", [10]));

// table_copy.wast:547
assert_return(() => call($7, "check_t1", [11]), 6);

// table_copy.wast:548
assert_return(() => call($7, "check_t1", [12]), 3);

// table_copy.wast:549
assert_return(() => call($7, "check_t1", [13]), 2);

// table_copy.wast:550
assert_return(() => call($7, "check_t1", [14]), 5);

// table_copy.wast:551
assert_return(() => call($7, "check_t1", [15]), 7);

// table_copy.wast:552
assert_trap(() => call($7, "check_t1", [16]));

// table_copy.wast:553
assert_trap(() => call($7, "check_t1", [17]));

// table_copy.wast:554
assert_trap(() => call($7, "check_t1", [18]));

// table_copy.wast:555
assert_trap(() => call($7, "check_t1", [19]));

// table_copy.wast:556
assert_trap(() => call($7, "check_t1", [20]));

// table_copy.wast:557
assert_trap(() => call($7, "check_t1", [21]));

// table_copy.wast:558
assert_trap(() => call($7, "check_t1", [22]));

// table_copy.wast:559
assert_trap(() => call($7, "check_t1", [23]));

// table_copy.wast:560
assert_trap(() => call($7, "check_t1", [24]));

// table_copy.wast:561
assert_trap(() => call($7, "check_t1", [25]));

// table_copy.wast:562
assert_trap(() => call($7, "check_t1", [26]));

// table_copy.wast:563
assert_trap(() => call($7, "check_t1", [27]));

// table_copy.wast:564
assert_trap(() => call($7, "check_t1", [28]));

// table_copy.wast:565
assert_trap(() => call($7, "check_t1", [29]));

// table_copy.wast:567
let $8 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x02\x01\x41\x03\x0b\x00\x04\x01\x03\x01\x04\x02\x01\x41\x0b\x0b\x00\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0a\x41\x0c\x41\x07\xfc\x0e\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b");

// table_copy.wast:597
run(() => call($8, "test", []));

// table_copy.wast:598
assert_trap(() => call($8, "check_t0", [0]));

// table_copy.wast:599
assert_trap(() => call($8, "check_t0", [1]));

// table_copy.wast:600
assert_return(() => call($8, "check_t0", [2]), 3);

// table_copy.wast:601
assert_return(() => call($8, "check_t0", [3]), 1);

// table_copy.wast:602
assert_return(() => call($8, "check_t0", [4]), 4);

// table_copy.wast:603
assert_return(() => call($8, "check_t0", [5]), 1);

// table_copy.wast:604
assert_trap(() => call($8, "check_t0", [6]));

// table_copy.wast:605
assert_trap(() => call($8, "check_t0", [7]));

// table_copy.wast:606
assert_trap(() => call($8, "check_t0", [8]));

// table_copy.wast:607
assert_trap(() => call($8, "check_t0", [9]));

// table_copy.wast:608
assert_return(() => call($8, "check_t0", [10]), 7);

// table_copy.wast:609
assert_return(() => call($8, "check_t0", [11]), 5);

// table_copy.wast:610
assert_return(() => call($8, "check_t0", [12]), 2);

// table_copy.wast:611
assert_return(() => call($8, "check_t0", [13]), 3);

// table_copy.wast:612
assert_return(() => call($8, "check_t0", [14]), 6);

// table_copy.wast:613
assert_trap(() => call($8, "check_t0", [15]));

// table_copy.wast:614
assert_trap(() => call($8, "check_t0", [16]));

// table_copy.wast:615
assert_trap(() => call($8, "check_t0", [17]));

// table_copy.wast:616
assert_trap(() => call($8, "check_t0", [18]));

// table_copy.wast:617
assert_trap(() => call($8, "check_t0", [19]));

// table_copy.wast:618
assert_trap(() => call($8, "check_t0", [20]));

// table_copy.wast:619
assert_trap(() => call($8, "check_t0", [21]));

// table_copy.wast:620
assert_trap(() => call($8, "check_t0", [22]));

// table_copy.wast:621
assert_trap(() => call($8, "check_t0", [23]));

// table_copy.wast:622
assert_trap(() => call($8, "check_t0", [24]));

// table_copy.wast:623
assert_trap(() => call($8, "check_t0", [25]));

// table_copy.wast:624
assert_trap(() => call($8, "check_t0", [26]));

// table_copy.wast:625
assert_trap(() => call($8, "check_t0", [27]));

// table_copy.wast:626
assert_trap(() => call($8, "check_t0", [28]));

// table_copy.wast:627
assert_trap(() => call($8, "check_t0", [29]));

// table_copy.wast:628
assert_trap(() => call($8, "check_t1", [0]));

// table_copy.wast:629
assert_trap(() => call($8, "check_t1", [1]));

// table_copy.wast:630
assert_trap(() => call($8, "check_t1", [2]));

// table_copy.wast:631
assert_return(() => call($8, "check_t1", [3]), 1);

// table_copy.wast:632
assert_return(() => call($8, "check_t1", [4]), 3);

// table_copy.wast:633
assert_return(() => call($8, "check_t1", [5]), 1);

// table_copy.wast:634
assert_return(() => call($8, "check_t1", [6]), 4);

// table_copy.wast:635
assert_trap(() => call($8, "check_t1", [7]));

// table_copy.wast:636
assert_trap(() => call($8, "check_t1", [8]));

// table_copy.wast:637
assert_trap(() => call($8, "check_t1", [9]));

// table_copy.wast:638
assert_trap(() => call($8, "check_t1", [10]));

// table_copy.wast:639
assert_return(() => call($8, "check_t1", [11]), 6);

// table_copy.wast:640
assert_return(() => call($8, "check_t1", [12]), 3);

// table_copy.wast:641
assert_return(() => call($8, "check_t1", [13]), 2);

// table_copy.wast:642
assert_return(() => call($8, "check_t1", [14]), 5);

// table_copy.wast:643
assert_return(() => call($8, "check_t1", [15]), 7);

// table_copy.wast:644
assert_trap(() => call($8, "check_t1", [16]));

// table_copy.wast:645
assert_trap(() => call($8, "check_t1", [17]));

// table_copy.wast:646
assert_trap(() => call($8, "check_t1", [18]));

// table_copy.wast:647
assert_trap(() => call($8, "check_t1", [19]));

// table_copy.wast:648
assert_trap(() => call($8, "check_t1", [20]));

// table_copy.wast:649
assert_trap(() => call($8, "check_t1", [21]));

// table_copy.wast:650
assert_trap(() => call($8, "check_t1", [22]));

// table_copy.wast:651
assert_trap(() => call($8, "check_t1", [23]));

// table_copy.wast:652
assert_trap(() => call($8, "check_t1", [24]));

// table_copy.wast:653
assert_trap(() => call($8, "check_t1", [25]));

// table_copy.wast:654
assert_trap(() => call($8, "check_t1", [26]));

// table_copy.wast:655
assert_trap(() => call($8, "check_t1", [27]));

// table_copy.wast:656
assert_trap(() => call($8, "check_t1", [28]));

// table_copy.wast:657
assert_trap(() => call($8, "check_t1", [29]));

// table_copy.wast:659
let $9 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x02\x01\x41\x03\x0b\x00\x04\x01\x03\x01\x04\x02\x01\x41\x0b\x0b\x00\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0c\x41\x0a\x41\x07\xfc\x0e\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b");

// table_copy.wast:689
run(() => call($9, "test", []));

// table_copy.wast:690
assert_trap(() => call($9, "check_t0", [0]));

// table_copy.wast:691
assert_trap(() => call($9, "check_t0", [1]));

// table_copy.wast:692
assert_return(() => call($9, "check_t0", [2]), 3);

// table_copy.wast:693
assert_return(() => call($9, "check_t0", [3]), 1);

// table_copy.wast:694
assert_return(() => call($9, "check_t0", [4]), 4);

// table_copy.wast:695
assert_return(() => call($9, "check_t0", [5]), 1);

// table_copy.wast:696
assert_trap(() => call($9, "check_t0", [6]));

// table_copy.wast:697
assert_trap(() => call($9, "check_t0", [7]));

// table_copy.wast:698
assert_trap(() => call($9, "check_t0", [8]));

// table_copy.wast:699
assert_trap(() => call($9, "check_t0", [9]));

// table_copy.wast:700
assert_trap(() => call($9, "check_t0", [10]));

// table_copy.wast:701
assert_trap(() => call($9, "check_t0", [11]));

// table_copy.wast:702
assert_trap(() => call($9, "check_t0", [12]));

// table_copy.wast:703
assert_trap(() => call($9, "check_t0", [13]));

// table_copy.wast:704
assert_return(() => call($9, "check_t0", [14]), 7);

// table_copy.wast:705
assert_return(() => call($9, "check_t0", [15]), 5);

// table_copy.wast:706
assert_return(() => call($9, "check_t0", [16]), 2);

// table_copy.wast:707
assert_return(() => call($9, "check_t0", [17]), 3);

// table_copy.wast:708
assert_return(() => call($9, "check_t0", [18]), 6);

// table_copy.wast:709
assert_trap(() => call($9, "check_t0", [19]));

// table_copy.wast:710
assert_trap(() => call($9, "check_t0", [20]));

// table_copy.wast:711
assert_trap(() => call($9, "check_t0", [21]));

// table_copy.wast:712
assert_trap(() => call($9, "check_t0", [22]));

// table_copy.wast:713
assert_trap(() => call($9, "check_t0", [23]));

// table_copy.wast:714
assert_trap(() => call($9, "check_t0", [24]));

// table_copy.wast:715
assert_trap(() => call($9, "check_t0", [25]));

// table_copy.wast:716
assert_trap(() => call($9, "check_t0", [26]));

// table_copy.wast:717
assert_trap(() => call($9, "check_t0", [27]));

// table_copy.wast:718
assert_trap(() => call($9, "check_t0", [28]));

// table_copy.wast:719
assert_trap(() => call($9, "check_t0", [29]));

// table_copy.wast:720
assert_trap(() => call($9, "check_t1", [0]));

// table_copy.wast:721
assert_trap(() => call($9, "check_t1", [1]));

// table_copy.wast:722
assert_trap(() => call($9, "check_t1", [2]));

// table_copy.wast:723
assert_return(() => call($9, "check_t1", [3]), 1);

// table_copy.wast:724
assert_return(() => call($9, "check_t1", [4]), 3);

// table_copy.wast:725
assert_return(() => call($9, "check_t1", [5]), 1);

// table_copy.wast:726
assert_return(() => call($9, "check_t1", [6]), 4);

// table_copy.wast:727
assert_trap(() => call($9, "check_t1", [7]));

// table_copy.wast:728
assert_trap(() => call($9, "check_t1", [8]));

// table_copy.wast:729
assert_trap(() => call($9, "check_t1", [9]));

// table_copy.wast:730
assert_trap(() => call($9, "check_t1", [10]));

// table_copy.wast:731
assert_return(() => call($9, "check_t1", [11]), 6);

// table_copy.wast:732
assert_return(() => call($9, "check_t1", [12]), 3);

// table_copy.wast:733
assert_return(() => call($9, "check_t1", [13]), 2);

// table_copy.wast:734
assert_return(() => call($9, "check_t1", [14]), 5);

// table_copy.wast:735
assert_return(() => call($9, "check_t1", [15]), 7);

// table_copy.wast:736
assert_trap(() => call($9, "check_t1", [16]));

// table_copy.wast:737
assert_trap(() => call($9, "check_t1", [17]));

// table_copy.wast:738
assert_trap(() => call($9, "check_t1", [18]));

// table_copy.wast:739
assert_trap(() => call($9, "check_t1", [19]));

// table_copy.wast:740
assert_trap(() => call($9, "check_t1", [20]));

// table_copy.wast:741
assert_trap(() => call($9, "check_t1", [21]));

// table_copy.wast:742
assert_trap(() => call($9, "check_t1", [22]));

// table_copy.wast:743
assert_trap(() => call($9, "check_t1", [23]));

// table_copy.wast:744
assert_trap(() => call($9, "check_t1", [24]));

// table_copy.wast:745
assert_trap(() => call($9, "check_t1", [25]));

// table_copy.wast:746
assert_trap(() => call($9, "check_t1", [26]));

// table_copy.wast:747
assert_trap(() => call($9, "check_t1", [27]));

// table_copy.wast:748
assert_trap(() => call($9, "check_t1", [28]));

// table_copy.wast:749
assert_trap(() => call($9, "check_t1", [29]));

// table_copy.wast:751
let $10 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x02\x01\x41\x03\x0b\x00\x04\x01\x03\x01\x04\x02\x01\x41\x0b\x0b\x00\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0a\x41\x00\x41\x14\xfc\x0e\x01\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b");

// table_copy.wast:781
run(() => call($10, "test", []));

// table_copy.wast:782
assert_trap(() => call($10, "check_t0", [0]));

// table_copy.wast:783
assert_trap(() => call($10, "check_t0", [1]));

// table_copy.wast:784
assert_return(() => call($10, "check_t0", [2]), 3);

// table_copy.wast:785
assert_return(() => call($10, "check_t0", [3]), 1);

// table_copy.wast:786
assert_return(() => call($10, "check_t0", [4]), 4);

// table_copy.wast:787
assert_return(() => call($10, "check_t0", [5]), 1);

// table_copy.wast:788
assert_trap(() => call($10, "check_t0", [6]));

// table_copy.wast:789
assert_trap(() => call($10, "check_t0", [7]));

// table_copy.wast:790
assert_trap(() => call($10, "check_t0", [8]));

// table_copy.wast:791
assert_trap(() => call($10, "check_t0", [9]));

// table_copy.wast:792
assert_trap(() => call($10, "check_t0", [10]));

// table_copy.wast:793
assert_trap(() => call($10, "check_t0", [11]));

// table_copy.wast:794
assert_return(() => call($10, "check_t0", [12]), 7);

// table_copy.wast:795
assert_return(() => call($10, "check_t0", [13]), 5);

// table_copy.wast:796
assert_return(() => call($10, "check_t0", [14]), 2);

// table_copy.wast:797
assert_return(() => call($10, "check_t0", [15]), 3);

// table_copy.wast:798
assert_return(() => call($10, "check_t0", [16]), 6);

// table_copy.wast:799
assert_trap(() => call($10, "check_t0", [17]));

// table_copy.wast:800
assert_trap(() => call($10, "check_t0", [18]));

// table_copy.wast:801
assert_trap(() => call($10, "check_t0", [19]));

// table_copy.wast:802
assert_trap(() => call($10, "check_t0", [20]));

// table_copy.wast:803
assert_trap(() => call($10, "check_t0", [21]));

// table_copy.wast:804
assert_trap(() => call($10, "check_t0", [22]));

// table_copy.wast:805
assert_trap(() => call($10, "check_t0", [23]));

// table_copy.wast:806
assert_trap(() => call($10, "check_t0", [24]));

// table_copy.wast:807
assert_trap(() => call($10, "check_t0", [25]));

// table_copy.wast:808
assert_trap(() => call($10, "check_t0", [26]));

// table_copy.wast:809
assert_trap(() => call($10, "check_t0", [27]));

// table_copy.wast:810
assert_trap(() => call($10, "check_t0", [28]));

// table_copy.wast:811
assert_trap(() => call($10, "check_t0", [29]));

// table_copy.wast:812
assert_trap(() => call($10, "check_t1", [0]));

// table_copy.wast:813
assert_trap(() => call($10, "check_t1", [1]));

// table_copy.wast:814
assert_trap(() => call($10, "check_t1", [2]));

// table_copy.wast:815
assert_return(() => call($10, "check_t1", [3]), 1);

// table_copy.wast:816
assert_return(() => call($10, "check_t1", [4]), 3);

// table_copy.wast:817
assert_return(() => call($10, "check_t1", [5]), 1);

// table_copy.wast:818
assert_return(() => call($10, "check_t1", [6]), 4);

// table_copy.wast:819
assert_trap(() => call($10, "check_t1", [7]));

// table_copy.wast:820
assert_trap(() => call($10, "check_t1", [8]));

// table_copy.wast:821
assert_trap(() => call($10, "check_t1", [9]));

// table_copy.wast:822
assert_trap(() => call($10, "check_t1", [10]));

// table_copy.wast:823
assert_trap(() => call($10, "check_t1", [11]));

// table_copy.wast:824
assert_return(() => call($10, "check_t1", [12]), 3);

// table_copy.wast:825
assert_return(() => call($10, "check_t1", [13]), 1);

// table_copy.wast:826
assert_return(() => call($10, "check_t1", [14]), 4);

// table_copy.wast:827
assert_return(() => call($10, "check_t1", [15]), 1);

// table_copy.wast:828
assert_trap(() => call($10, "check_t1", [16]));

// table_copy.wast:829
assert_trap(() => call($10, "check_t1", [17]));

// table_copy.wast:830
assert_trap(() => call($10, "check_t1", [18]));

// table_copy.wast:831
assert_trap(() => call($10, "check_t1", [19]));

// table_copy.wast:832
assert_trap(() => call($10, "check_t1", [20]));

// table_copy.wast:833
assert_trap(() => call($10, "check_t1", [21]));

// table_copy.wast:834
assert_return(() => call($10, "check_t1", [22]), 7);

// table_copy.wast:835
assert_return(() => call($10, "check_t1", [23]), 5);

// table_copy.wast:836
assert_return(() => call($10, "check_t1", [24]), 2);

// table_copy.wast:837
assert_return(() => call($10, "check_t1", [25]), 3);

// table_copy.wast:838
assert_return(() => call($10, "check_t1", [26]), 6);

// table_copy.wast:839
assert_trap(() => call($10, "check_t1", [27]));

// table_copy.wast:840
assert_trap(() => call($10, "check_t1", [28]));

// table_copy.wast:841
assert_trap(() => call($10, "check_t1", [29]));

// table_copy.wast:843
let $11 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x02\x01\x41\x02\x0b\x00\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x02\x01\x41\x0c\x0b\x00\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x00\x41\x03\x0b\x04\x01\x03\x01\x04\x00\x41\x0b\x0b\x05\x06\x03\x02\x05\x07\x0a\xce\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x83\x80\x80\x80\x00\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b");

// table_copy.wast:873
run(() => call($11, "test", []));

// table_copy.wast:874
assert_trap(() => call($11, "check_t0", [0]));

// table_copy.wast:875
assert_trap(() => call($11, "check_t0", [1]));

// table_copy.wast:876
assert_return(() => call($11, "check_t0", [2]), 3);

// table_copy.wast:877
assert_return(() => call($11, "check_t0", [3]), 1);

// table_copy.wast:878
assert_return(() => call($11, "check_t0", [4]), 4);

// table_copy.wast:879
assert_return(() => call($11, "check_t0", [5]), 1);

// table_copy.wast:880
assert_trap(() => call($11, "check_t0", [6]));

// table_copy.wast:881
assert_trap(() => call($11, "check_t0", [7]));

// table_copy.wast:882
assert_trap(() => call($11, "check_t0", [8]));

// table_copy.wast:883
assert_trap(() => call($11, "check_t0", [9]));

// table_copy.wast:884
assert_trap(() => call($11, "check_t0", [10]));

// table_copy.wast:885
assert_trap(() => call($11, "check_t0", [11]));

// table_copy.wast:886
assert_return(() => call($11, "check_t0", [12]), 7);

// table_copy.wast:887
assert_return(() => call($11, "check_t0", [13]), 5);

// table_copy.wast:888
assert_return(() => call($11, "check_t0", [14]), 2);

// table_copy.wast:889
assert_return(() => call($11, "check_t0", [15]), 3);

// table_copy.wast:890
assert_return(() => call($11, "check_t0", [16]), 6);

// table_copy.wast:891
assert_trap(() => call($11, "check_t0", [17]));

// table_copy.wast:892
assert_trap(() => call($11, "check_t0", [18]));

// table_copy.wast:893
assert_trap(() => call($11, "check_t0", [19]));

// table_copy.wast:894
assert_trap(() => call($11, "check_t0", [20]));

// table_copy.wast:895
assert_trap(() => call($11, "check_t0", [21]));

// table_copy.wast:896
assert_trap(() => call($11, "check_t0", [22]));

// table_copy.wast:897
assert_trap(() => call($11, "check_t0", [23]));

// table_copy.wast:898
assert_trap(() => call($11, "check_t0", [24]));

// table_copy.wast:899
assert_trap(() => call($11, "check_t0", [25]));

// table_copy.wast:900
assert_trap(() => call($11, "check_t0", [26]));

// table_copy.wast:901
assert_trap(() => call($11, "check_t0", [27]));

// table_copy.wast:902
assert_trap(() => call($11, "check_t0", [28]));

// table_copy.wast:903
assert_trap(() => call($11, "check_t0", [29]));

// table_copy.wast:904
assert_trap(() => call($11, "check_t1", [0]));

// table_copy.wast:905
assert_trap(() => call($11, "check_t1", [1]));

// table_copy.wast:906
assert_trap(() => call($11, "check_t1", [2]));

// table_copy.wast:907
assert_return(() => call($11, "check_t1", [3]), 1);

// table_copy.wast:908
assert_return(() => call($11, "check_t1", [4]), 3);

// table_copy.wast:909
assert_return(() => call($11, "check_t1", [5]), 1);

// table_copy.wast:910
assert_return(() => call($11, "check_t1", [6]), 4);

// table_copy.wast:911
assert_trap(() => call($11, "check_t1", [7]));

// table_copy.wast:912
assert_trap(() => call($11, "check_t1", [8]));

// table_copy.wast:913
assert_trap(() => call($11, "check_t1", [9]));

// table_copy.wast:914
assert_trap(() => call($11, "check_t1", [10]));

// table_copy.wast:915
assert_return(() => call($11, "check_t1", [11]), 6);

// table_copy.wast:916
assert_return(() => call($11, "check_t1", [12]), 3);

// table_copy.wast:917
assert_return(() => call($11, "check_t1", [13]), 2);

// table_copy.wast:918
assert_return(() => call($11, "check_t1", [14]), 5);

// table_copy.wast:919
assert_return(() => call($11, "check_t1", [15]), 7);

// table_copy.wast:920
assert_trap(() => call($11, "check_t1", [16]));

// table_copy.wast:921
assert_trap(() => call($11, "check_t1", [17]));

// table_copy.wast:922
assert_trap(() => call($11, "check_t1", [18]));

// table_copy.wast:923
assert_trap(() => call($11, "check_t1", [19]));

// table_copy.wast:924
assert_trap(() => call($11, "check_t1", [20]));

// table_copy.wast:925
assert_trap(() => call($11, "check_t1", [21]));

// table_copy.wast:926
assert_trap(() => call($11, "check_t1", [22]));

// table_copy.wast:927
assert_trap(() => call($11, "check_t1", [23]));

// table_copy.wast:928
assert_trap(() => call($11, "check_t1", [24]));

// table_copy.wast:929
assert_trap(() => call($11, "check_t1", [25]));

// table_copy.wast:930
assert_trap(() => call($11, "check_t1", [26]));

// table_copy.wast:931
assert_trap(() => call($11, "check_t1", [27]));

// table_copy.wast:932
assert_trap(() => call($11, "check_t1", [28]));

// table_copy.wast:933
assert_trap(() => call($11, "check_t1", [29]));

// table_copy.wast:935
let $12 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x02\x01\x41\x02\x0b\x00\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x02\x01\x41\x0c\x0b\x00\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x00\x41\x03\x0b\x04\x01\x03\x01\x04\x00\x41\x0b\x0b\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0d\x41\x02\x41\x03\xfc\x0e\x01\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b");

// table_copy.wast:965
run(() => call($12, "test", []));

// table_copy.wast:966
assert_trap(() => call($12, "check_t0", [0]));

// table_copy.wast:967
assert_trap(() => call($12, "check_t0", [1]));

// table_copy.wast:968
assert_return(() => call($12, "check_t0", [2]), 3);

// table_copy.wast:969
assert_return(() => call($12, "check_t0", [3]), 1);

// table_copy.wast:970
assert_return(() => call($12, "check_t0", [4]), 4);

// table_copy.wast:971
assert_return(() => call($12, "check_t0", [5]), 1);

// table_copy.wast:972
assert_trap(() => call($12, "check_t0", [6]));

// table_copy.wast:973
assert_trap(() => call($12, "check_t0", [7]));

// table_copy.wast:974
assert_trap(() => call($12, "check_t0", [8]));

// table_copy.wast:975
assert_trap(() => call($12, "check_t0", [9]));

// table_copy.wast:976
assert_trap(() => call($12, "check_t0", [10]));

// table_copy.wast:977
assert_trap(() => call($12, "check_t0", [11]));

// table_copy.wast:978
assert_return(() => call($12, "check_t0", [12]), 7);

// table_copy.wast:979
assert_return(() => call($12, "check_t0", [13]), 3);

// table_copy.wast:980
assert_return(() => call($12, "check_t0", [14]), 1);

// table_copy.wast:981
assert_return(() => call($12, "check_t0", [15]), 4);

// table_copy.wast:982
assert_return(() => call($12, "check_t0", [16]), 6);

// table_copy.wast:983
assert_trap(() => call($12, "check_t0", [17]));

// table_copy.wast:984
assert_trap(() => call($12, "check_t0", [18]));

// table_copy.wast:985
assert_trap(() => call($12, "check_t0", [19]));

// table_copy.wast:986
assert_trap(() => call($12, "check_t0", [20]));

// table_copy.wast:987
assert_trap(() => call($12, "check_t0", [21]));

// table_copy.wast:988
assert_trap(() => call($12, "check_t0", [22]));

// table_copy.wast:989
assert_trap(() => call($12, "check_t0", [23]));

// table_copy.wast:990
assert_trap(() => call($12, "check_t0", [24]));

// table_copy.wast:991
assert_trap(() => call($12, "check_t0", [25]));

// table_copy.wast:992
assert_trap(() => call($12, "check_t0", [26]));

// table_copy.wast:993
assert_trap(() => call($12, "check_t0", [27]));

// table_copy.wast:994
assert_trap(() => call($12, "check_t0", [28]));

// table_copy.wast:995
assert_trap(() => call($12, "check_t0", [29]));

// table_copy.wast:996
assert_trap(() => call($12, "check_t1", [0]));

// table_copy.wast:997
assert_trap(() => call($12, "check_t1", [1]));

// table_copy.wast:998
assert_trap(() => call($12, "check_t1", [2]));

// table_copy.wast:999
assert_return(() => call($12, "check_t1", [3]), 1);

// table_copy.wast:1000
assert_return(() => call($12, "check_t1", [4]), 3);

// table_copy.wast:1001
assert_return(() => call($12, "check_t1", [5]), 1);

// table_copy.wast:1002
assert_return(() => call($12, "check_t1", [6]), 4);

// table_copy.wast:1003
assert_trap(() => call($12, "check_t1", [7]));

// table_copy.wast:1004
assert_trap(() => call($12, "check_t1", [8]));

// table_copy.wast:1005
assert_trap(() => call($12, "check_t1", [9]));

// table_copy.wast:1006
assert_trap(() => call($12, "check_t1", [10]));

// table_copy.wast:1007
assert_return(() => call($12, "check_t1", [11]), 6);

// table_copy.wast:1008
assert_return(() => call($12, "check_t1", [12]), 3);

// table_copy.wast:1009
assert_return(() => call($12, "check_t1", [13]), 2);

// table_copy.wast:1010
assert_return(() => call($12, "check_t1", [14]), 5);

// table_copy.wast:1011
assert_return(() => call($12, "check_t1", [15]), 7);

// table_copy.wast:1012
assert_trap(() => call($12, "check_t1", [16]));

// table_copy.wast:1013
assert_trap(() => call($12, "check_t1", [17]));

// table_copy.wast:1014
assert_trap(() => call($12, "check_t1", [18]));

// table_copy.wast:1015
assert_trap(() => call($12, "check_t1", [19]));

// table_copy.wast:1016
assert_trap(() => call($12, "check_t1", [20]));

// table_copy.wast:1017
assert_trap(() => call($12, "check_t1", [21]));

// table_copy.wast:1018
assert_trap(() => call($12, "check_t1", [22]));

// table_copy.wast:1019
assert_trap(() => call($12, "check_t1", [23]));

// table_copy.wast:1020
assert_trap(() => call($12, "check_t1", [24]));

// table_copy.wast:1021
assert_trap(() => call($12, "check_t1", [25]));

// table_copy.wast:1022
assert_trap(() => call($12, "check_t1", [26]));

// table_copy.wast:1023
assert_trap(() => call($12, "check_t1", [27]));

// table_copy.wast:1024
assert_trap(() => call($12, "check_t1", [28]));

// table_copy.wast:1025
assert_trap(() => call($12, "check_t1", [29]));

// table_copy.wast:1027
let $13 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x02\x01\x41\x02\x0b\x00\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x02\x01\x41\x0c\x0b\x00\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x00\x41\x03\x0b\x04\x01\x03\x01\x04\x00\x41\x0b\x0b\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x19\x41\x0f\x41\x02\xfc\x0e\x01\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b");

// table_copy.wast:1057
run(() => call($13, "test", []));

// table_copy.wast:1058
assert_trap(() => call($13, "check_t0", [0]));

// table_copy.wast:1059
assert_trap(() => call($13, "check_t0", [1]));

// table_copy.wast:1060
assert_return(() => call($13, "check_t0", [2]), 3);

// table_copy.wast:1061
assert_return(() => call($13, "check_t0", [3]), 1);

// table_copy.wast:1062
assert_return(() => call($13, "check_t0", [4]), 4);

// table_copy.wast:1063
assert_return(() => call($13, "check_t0", [5]), 1);

// table_copy.wast:1064
assert_trap(() => call($13, "check_t0", [6]));

// table_copy.wast:1065
assert_trap(() => call($13, "check_t0", [7]));

// table_copy.wast:1066
assert_trap(() => call($13, "check_t0", [8]));

// table_copy.wast:1067
assert_trap(() => call($13, "check_t0", [9]));

// table_copy.wast:1068
assert_trap(() => call($13, "check_t0", [10]));

// table_copy.wast:1069
assert_trap(() => call($13, "check_t0", [11]));

// table_copy.wast:1070
assert_return(() => call($13, "check_t0", [12]), 7);

// table_copy.wast:1071
assert_return(() => call($13, "check_t0", [13]), 5);

// table_copy.wast:1072
assert_return(() => call($13, "check_t0", [14]), 2);

// table_copy.wast:1073
assert_return(() => call($13, "check_t0", [15]), 3);

// table_copy.wast:1074
assert_return(() => call($13, "check_t0", [16]), 6);

// table_copy.wast:1075
assert_trap(() => call($13, "check_t0", [17]));

// table_copy.wast:1076
assert_trap(() => call($13, "check_t0", [18]));

// table_copy.wast:1077
assert_trap(() => call($13, "check_t0", [19]));

// table_copy.wast:1078
assert_trap(() => call($13, "check_t0", [20]));

// table_copy.wast:1079
assert_trap(() => call($13, "check_t0", [21]));

// table_copy.wast:1080
assert_trap(() => call($13, "check_t0", [22]));

// table_copy.wast:1081
assert_trap(() => call($13, "check_t0", [23]));

// table_copy.wast:1082
assert_trap(() => call($13, "check_t0", [24]));

// table_copy.wast:1083
assert_return(() => call($13, "check_t0", [25]), 3);

// table_copy.wast:1084
assert_return(() => call($13, "check_t0", [26]), 6);

// table_copy.wast:1085
assert_trap(() => call($13, "check_t0", [27]));

// table_copy.wast:1086
assert_trap(() => call($13, "check_t0", [28]));

// table_copy.wast:1087
assert_trap(() => call($13, "check_t0", [29]));

// table_copy.wast:1088
assert_trap(() => call($13, "check_t1", [0]));

// table_copy.wast:1089
assert_trap(() => call($13, "check_t1", [1]));

// table_copy.wast:1090
assert_trap(() => call($13, "check_t1", [2]));

// table_copy.wast:1091
assert_return(() => call($13, "check_t1", [3]), 1);

// table_copy.wast:1092
assert_return(() => call($13, "check_t1", [4]), 3);

// table_copy.wast:1093
assert_return(() => call($13, "check_t1", [5]), 1);

// table_copy.wast:1094
assert_return(() => call($13, "check_t1", [6]), 4);

// table_copy.wast:1095
assert_trap(() => call($13, "check_t1", [7]));

// table_copy.wast:1096
assert_trap(() => call($13, "check_t1", [8]));

// table_copy.wast:1097
assert_trap(() => call($13, "check_t1", [9]));

// table_copy.wast:1098
assert_trap(() => call($13, "check_t1", [10]));

// table_copy.wast:1099
assert_return(() => call($13, "check_t1", [11]), 6);

// table_copy.wast:1100
assert_return(() => call($13, "check_t1", [12]), 3);

// table_copy.wast:1101
assert_return(() => call($13, "check_t1", [13]), 2);

// table_copy.wast:1102
assert_return(() => call($13, "check_t1", [14]), 5);

// table_copy.wast:1103
assert_return(() => call($13, "check_t1", [15]), 7);

// table_copy.wast:1104
assert_trap(() => call($13, "check_t1", [16]));

// table_copy.wast:1105
assert_trap(() => call($13, "check_t1", [17]));

// table_copy.wast:1106
assert_trap(() => call($13, "check_t1", [18]));

// table_copy.wast:1107
assert_trap(() => call($13, "check_t1", [19]));

// table_copy.wast:1108
assert_trap(() => call($13, "check_t1", [20]));

// table_copy.wast:1109
assert_trap(() => call($13, "check_t1", [21]));

// table_copy.wast:1110
assert_trap(() => call($13, "check_t1", [22]));

// table_copy.wast:1111
assert_trap(() => call($13, "check_t1", [23]));

// table_copy.wast:1112
assert_trap(() => call($13, "check_t1", [24]));

// table_copy.wast:1113
assert_trap(() => call($13, "check_t1", [25]));

// table_copy.wast:1114
assert_trap(() => call($13, "check_t1", [26]));

// table_copy.wast:1115
assert_trap(() => call($13, "check_t1", [27]));

// table_copy.wast:1116
assert_trap(() => call($13, "check_t1", [28]));

// table_copy.wast:1117
assert_trap(() => call($13, "check_t1", [29]));

// table_copy.wast:1119
let $14 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x02\x01\x41\x02\x0b\x00\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x02\x01\x41\x0c\x0b\x00\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x00\x41\x03\x0b\x04\x01\x03\x01\x04\x00\x41\x0b\x0b\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0d\x41\x19\x41\x03\xfc\x0e\x01\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b");

// table_copy.wast:1149
run(() => call($14, "test", []));

// table_copy.wast:1150
assert_trap(() => call($14, "check_t0", [0]));

// table_copy.wast:1151
assert_trap(() => call($14, "check_t0", [1]));

// table_copy.wast:1152
assert_return(() => call($14, "check_t0", [2]), 3);

// table_copy.wast:1153
assert_return(() => call($14, "check_t0", [3]), 1);

// table_copy.wast:1154
assert_return(() => call($14, "check_t0", [4]), 4);

// table_copy.wast:1155
assert_return(() => call($14, "check_t0", [5]), 1);

// table_copy.wast:1156
assert_trap(() => call($14, "check_t0", [6]));

// table_copy.wast:1157
assert_trap(() => call($14, "check_t0", [7]));

// table_copy.wast:1158
assert_trap(() => call($14, "check_t0", [8]));

// table_copy.wast:1159
assert_trap(() => call($14, "check_t0", [9]));

// table_copy.wast:1160
assert_trap(() => call($14, "check_t0", [10]));

// table_copy.wast:1161
assert_trap(() => call($14, "check_t0", [11]));

// table_copy.wast:1162
assert_return(() => call($14, "check_t0", [12]), 7);

// table_copy.wast:1163
assert_trap(() => call($14, "check_t0", [13]));

// table_copy.wast:1164
assert_trap(() => call($14, "check_t0", [14]));

// table_copy.wast:1165
assert_trap(() => call($14, "check_t0", [15]));

// table_copy.wast:1166
assert_return(() => call($14, "check_t0", [16]), 6);

// table_copy.wast:1167
assert_trap(() => call($14, "check_t0", [17]));

// table_copy.wast:1168
assert_trap(() => call($14, "check_t0", [18]));

// table_copy.wast:1169
assert_trap(() => call($14, "check_t0", [19]));

// table_copy.wast:1170
assert_trap(() => call($14, "check_t0", [20]));

// table_copy.wast:1171
assert_trap(() => call($14, "check_t0", [21]));

// table_copy.wast:1172
assert_trap(() => call($14, "check_t0", [22]));

// table_copy.wast:1173
assert_trap(() => call($14, "check_t0", [23]));

// table_copy.wast:1174
assert_trap(() => call($14, "check_t0", [24]));

// table_copy.wast:1175
assert_trap(() => call($14, "check_t0", [25]));

// table_copy.wast:1176
assert_trap(() => call($14, "check_t0", [26]));

// table_copy.wast:1177
assert_trap(() => call($14, "check_t0", [27]));

// table_copy.wast:1178
assert_trap(() => call($14, "check_t0", [28]));

// table_copy.wast:1179
assert_trap(() => call($14, "check_t0", [29]));

// table_copy.wast:1180
assert_trap(() => call($14, "check_t1", [0]));

// table_copy.wast:1181
assert_trap(() => call($14, "check_t1", [1]));

// table_copy.wast:1182
assert_trap(() => call($14, "check_t1", [2]));

// table_copy.wast:1183
assert_return(() => call($14, "check_t1", [3]), 1);

// table_copy.wast:1184
assert_return(() => call($14, "check_t1", [4]), 3);

// table_copy.wast:1185
assert_return(() => call($14, "check_t1", [5]), 1);

// table_copy.wast:1186
assert_return(() => call($14, "check_t1", [6]), 4);

// table_copy.wast:1187
assert_trap(() => call($14, "check_t1", [7]));

// table_copy.wast:1188
assert_trap(() => call($14, "check_t1", [8]));

// table_copy.wast:1189
assert_trap(() => call($14, "check_t1", [9]));

// table_copy.wast:1190
assert_trap(() => call($14, "check_t1", [10]));

// table_copy.wast:1191
assert_return(() => call($14, "check_t1", [11]), 6);

// table_copy.wast:1192
assert_return(() => call($14, "check_t1", [12]), 3);

// table_copy.wast:1193
assert_return(() => call($14, "check_t1", [13]), 2);

// table_copy.wast:1194
assert_return(() => call($14, "check_t1", [14]), 5);

// table_copy.wast:1195
assert_return(() => call($14, "check_t1", [15]), 7);

// table_copy.wast:1196
assert_trap(() => call($14, "check_t1", [16]));

// table_copy.wast:1197
assert_trap(() => call($14, "check_t1", [17]));

// table_copy.wast:1198
assert_trap(() => call($14, "check_t1", [18]));

// table_copy.wast:1199
assert_trap(() => call($14, "check_t1", [19]));

// table_copy.wast:1200
assert_trap(() => call($14, "check_t1", [20]));

// table_copy.wast:1201
assert_trap(() => call($14, "check_t1", [21]));

// table_copy.wast:1202
assert_trap(() => call($14, "check_t1", [22]));

// table_copy.wast:1203
assert_trap(() => call($14, "check_t1", [23]));

// table_copy.wast:1204
assert_trap(() => call($14, "check_t1", [24]));

// table_copy.wast:1205
assert_trap(() => call($14, "check_t1", [25]));

// table_copy.wast:1206
assert_trap(() => call($14, "check_t1", [26]));

// table_copy.wast:1207
assert_trap(() => call($14, "check_t1", [27]));

// table_copy.wast:1208
assert_trap(() => call($14, "check_t1", [28]));

// table_copy.wast:1209
assert_trap(() => call($14, "check_t1", [29]));

// table_copy.wast:1211
let $15 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x02\x01\x41\x02\x0b\x00\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x02\x01\x41\x0c\x0b\x00\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x00\x41\x03\x0b\x04\x01\x03\x01\x04\x00\x41\x0b\x0b\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x14\x41\x16\x41\x04\xfc\x0e\x01\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b");

// table_copy.wast:1241
run(() => call($15, "test", []));

// table_copy.wast:1242
assert_trap(() => call($15, "check_t0", [0]));

// table_copy.wast:1243
assert_trap(() => call($15, "check_t0", [1]));

// table_copy.wast:1244
assert_return(() => call($15, "check_t0", [2]), 3);

// table_copy.wast:1245
assert_return(() => call($15, "check_t0", [3]), 1);

// table_copy.wast:1246
assert_return(() => call($15, "check_t0", [4]), 4);

// table_copy.wast:1247
assert_return(() => call($15, "check_t0", [5]), 1);

// table_copy.wast:1248
assert_trap(() => call($15, "check_t0", [6]));

// table_copy.wast:1249
assert_trap(() => call($15, "check_t0", [7]));

// table_copy.wast:1250
assert_trap(() => call($15, "check_t0", [8]));

// table_copy.wast:1251
assert_trap(() => call($15, "check_t0", [9]));

// table_copy.wast:1252
assert_trap(() => call($15, "check_t0", [10]));

// table_copy.wast:1253
assert_trap(() => call($15, "check_t0", [11]));

// table_copy.wast:1254
assert_return(() => call($15, "check_t0", [12]), 7);

// table_copy.wast:1255
assert_return(() => call($15, "check_t0", [13]), 5);

// table_copy.wast:1256
assert_return(() => call($15, "check_t0", [14]), 2);

// table_copy.wast:1257
assert_return(() => call($15, "check_t0", [15]), 3);

// table_copy.wast:1258
assert_return(() => call($15, "check_t0", [16]), 6);

// table_copy.wast:1259
assert_trap(() => call($15, "check_t0", [17]));

// table_copy.wast:1260
assert_trap(() => call($15, "check_t0", [18]));

// table_copy.wast:1261
assert_trap(() => call($15, "check_t0", [19]));

// table_copy.wast:1262
assert_trap(() => call($15, "check_t0", [20]));

// table_copy.wast:1263
assert_trap(() => call($15, "check_t0", [21]));

// table_copy.wast:1264
assert_trap(() => call($15, "check_t0", [22]));

// table_copy.wast:1265
assert_trap(() => call($15, "check_t0", [23]));

// table_copy.wast:1266
assert_trap(() => call($15, "check_t0", [24]));

// table_copy.wast:1267
assert_trap(() => call($15, "check_t0", [25]));

// table_copy.wast:1268
assert_trap(() => call($15, "check_t0", [26]));

// table_copy.wast:1269
assert_trap(() => call($15, "check_t0", [27]));

// table_copy.wast:1270
assert_trap(() => call($15, "check_t0", [28]));

// table_copy.wast:1271
assert_trap(() => call($15, "check_t0", [29]));

// table_copy.wast:1272
assert_trap(() => call($15, "check_t1", [0]));

// table_copy.wast:1273
assert_trap(() => call($15, "check_t1", [1]));

// table_copy.wast:1274
assert_trap(() => call($15, "check_t1", [2]));

// table_copy.wast:1275
assert_return(() => call($15, "check_t1", [3]), 1);

// table_copy.wast:1276
assert_return(() => call($15, "check_t1", [4]), 3);

// table_copy.wast:1277
assert_return(() => call($15, "check_t1", [5]), 1);

// table_copy.wast:1278
assert_return(() => call($15, "check_t1", [6]), 4);

// table_copy.wast:1279
assert_trap(() => call($15, "check_t1", [7]));

// table_copy.wast:1280
assert_trap(() => call($15, "check_t1", [8]));

// table_copy.wast:1281
assert_trap(() => call($15, "check_t1", [9]));

// table_copy.wast:1282
assert_trap(() => call($15, "check_t1", [10]));

// table_copy.wast:1283
assert_return(() => call($15, "check_t1", [11]), 6);

// table_copy.wast:1284
assert_return(() => call($15, "check_t1", [12]), 3);

// table_copy.wast:1285
assert_return(() => call($15, "check_t1", [13]), 2);

// table_copy.wast:1286
assert_return(() => call($15, "check_t1", [14]), 5);

// table_copy.wast:1287
assert_return(() => call($15, "check_t1", [15]), 7);

// table_copy.wast:1288
assert_trap(() => call($15, "check_t1", [16]));

// table_copy.wast:1289
assert_trap(() => call($15, "check_t1", [17]));

// table_copy.wast:1290
assert_trap(() => call($15, "check_t1", [18]));

// table_copy.wast:1291
assert_trap(() => call($15, "check_t1", [19]));

// table_copy.wast:1292
assert_trap(() => call($15, "check_t1", [20]));

// table_copy.wast:1293
assert_trap(() => call($15, "check_t1", [21]));

// table_copy.wast:1294
assert_trap(() => call($15, "check_t1", [22]));

// table_copy.wast:1295
assert_trap(() => call($15, "check_t1", [23]));

// table_copy.wast:1296
assert_trap(() => call($15, "check_t1", [24]));

// table_copy.wast:1297
assert_trap(() => call($15, "check_t1", [25]));

// table_copy.wast:1298
assert_trap(() => call($15, "check_t1", [26]));

// table_copy.wast:1299
assert_trap(() => call($15, "check_t1", [27]));

// table_copy.wast:1300
assert_trap(() => call($15, "check_t1", [28]));

// table_copy.wast:1301
assert_trap(() => call($15, "check_t1", [29]));

// table_copy.wast:1303
let $16 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x02\x01\x41\x02\x0b\x00\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x02\x01\x41\x0c\x0b\x00\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x00\x41\x03\x0b\x04\x01\x03\x01\x04\x00\x41\x0b\x0b\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x19\x41\x01\x41\x03\xfc\x0e\x01\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b");

// table_copy.wast:1333
run(() => call($16, "test", []));

// table_copy.wast:1334
assert_trap(() => call($16, "check_t0", [0]));

// table_copy.wast:1335
assert_trap(() => call($16, "check_t0", [1]));

// table_copy.wast:1336
assert_return(() => call($16, "check_t0", [2]), 3);

// table_copy.wast:1337
assert_return(() => call($16, "check_t0", [3]), 1);

// table_copy.wast:1338
assert_return(() => call($16, "check_t0", [4]), 4);

// table_copy.wast:1339
assert_return(() => call($16, "check_t0", [5]), 1);

// table_copy.wast:1340
assert_trap(() => call($16, "check_t0", [6]));

// table_copy.wast:1341
assert_trap(() => call($16, "check_t0", [7]));

// table_copy.wast:1342
assert_trap(() => call($16, "check_t0", [8]));

// table_copy.wast:1343
assert_trap(() => call($16, "check_t0", [9]));

// table_copy.wast:1344
assert_trap(() => call($16, "check_t0", [10]));

// table_copy.wast:1345
assert_trap(() => call($16, "check_t0", [11]));

// table_copy.wast:1346
assert_return(() => call($16, "check_t0", [12]), 7);

// table_copy.wast:1347
assert_return(() => call($16, "check_t0", [13]), 5);

// table_copy.wast:1348
assert_return(() => call($16, "check_t0", [14]), 2);

// table_copy.wast:1349
assert_return(() => call($16, "check_t0", [15]), 3);

// table_copy.wast:1350
assert_return(() => call($16, "check_t0", [16]), 6);

// table_copy.wast:1351
assert_trap(() => call($16, "check_t0", [17]));

// table_copy.wast:1352
assert_trap(() => call($16, "check_t0", [18]));

// table_copy.wast:1353
assert_trap(() => call($16, "check_t0", [19]));

// table_copy.wast:1354
assert_trap(() => call($16, "check_t0", [20]));

// table_copy.wast:1355
assert_trap(() => call($16, "check_t0", [21]));

// table_copy.wast:1356
assert_trap(() => call($16, "check_t0", [22]));

// table_copy.wast:1357
assert_trap(() => call($16, "check_t0", [23]));

// table_copy.wast:1358
assert_trap(() => call($16, "check_t0", [24]));

// table_copy.wast:1359
assert_trap(() => call($16, "check_t0", [25]));

// table_copy.wast:1360
assert_return(() => call($16, "check_t0", [26]), 3);

// table_copy.wast:1361
assert_return(() => call($16, "check_t0", [27]), 1);

// table_copy.wast:1362
assert_trap(() => call($16, "check_t0", [28]));

// table_copy.wast:1363
assert_trap(() => call($16, "check_t0", [29]));

// table_copy.wast:1364
assert_trap(() => call($16, "check_t1", [0]));

// table_copy.wast:1365
assert_trap(() => call($16, "check_t1", [1]));

// table_copy.wast:1366
assert_trap(() => call($16, "check_t1", [2]));

// table_copy.wast:1367
assert_return(() => call($16, "check_t1", [3]), 1);

// table_copy.wast:1368
assert_return(() => call($16, "check_t1", [4]), 3);

// table_copy.wast:1369
assert_return(() => call($16, "check_t1", [5]), 1);

// table_copy.wast:1370
assert_return(() => call($16, "check_t1", [6]), 4);

// table_copy.wast:1371
assert_trap(() => call($16, "check_t1", [7]));

// table_copy.wast:1372
assert_trap(() => call($16, "check_t1", [8]));

// table_copy.wast:1373
assert_trap(() => call($16, "check_t1", [9]));

// table_copy.wast:1374
assert_trap(() => call($16, "check_t1", [10]));

// table_copy.wast:1375
assert_return(() => call($16, "check_t1", [11]), 6);

// table_copy.wast:1376
assert_return(() => call($16, "check_t1", [12]), 3);

// table_copy.wast:1377
assert_return(() => call($16, "check_t1", [13]), 2);

// table_copy.wast:1378
assert_return(() => call($16, "check_t1", [14]), 5);

// table_copy.wast:1379
assert_return(() => call($16, "check_t1", [15]), 7);

// table_copy.wast:1380
assert_trap(() => call($16, "check_t1", [16]));

// table_copy.wast:1381
assert_trap(() => call($16, "check_t1", [17]));

// table_copy.wast:1382
assert_trap(() => call($16, "check_t1", [18]));

// table_copy.wast:1383
assert_trap(() => call($16, "check_t1", [19]));

// table_copy.wast:1384
assert_trap(() => call($16, "check_t1", [20]));

// table_copy.wast:1385
assert_trap(() => call($16, "check_t1", [21]));

// table_copy.wast:1386
assert_trap(() => call($16, "check_t1", [22]));

// table_copy.wast:1387
assert_trap(() => call($16, "check_t1", [23]));

// table_copy.wast:1388
assert_trap(() => call($16, "check_t1", [24]));

// table_copy.wast:1389
assert_trap(() => call($16, "check_t1", [25]));

// table_copy.wast:1390
assert_trap(() => call($16, "check_t1", [26]));

// table_copy.wast:1391
assert_trap(() => call($16, "check_t1", [27]));

// table_copy.wast:1392
assert_trap(() => call($16, "check_t1", [28]));

// table_copy.wast:1393
assert_trap(() => call($16, "check_t1", [29]));

// table_copy.wast:1395
let $17 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x02\x01\x41\x02\x0b\x00\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x02\x01\x41\x0c\x0b\x00\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x00\x41\x03\x0b\x04\x01\x03\x01\x04\x00\x41\x0b\x0b\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0a\x41\x0c\x41\x07\xfc\x0e\x01\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b");

// table_copy.wast:1425
run(() => call($17, "test", []));

// table_copy.wast:1426
assert_trap(() => call($17, "check_t0", [0]));

// table_copy.wast:1427
assert_trap(() => call($17, "check_t0", [1]));

// table_copy.wast:1428
assert_return(() => call($17, "check_t0", [2]), 3);

// table_copy.wast:1429
assert_return(() => call($17, "check_t0", [3]), 1);

// table_copy.wast:1430
assert_return(() => call($17, "check_t0", [4]), 4);

// table_copy.wast:1431
assert_return(() => call($17, "check_t0", [5]), 1);

// table_copy.wast:1432
assert_trap(() => call($17, "check_t0", [6]));

// table_copy.wast:1433
assert_trap(() => call($17, "check_t0", [7]));

// table_copy.wast:1434
assert_trap(() => call($17, "check_t0", [8]));

// table_copy.wast:1435
assert_trap(() => call($17, "check_t0", [9]));

// table_copy.wast:1436
assert_return(() => call($17, "check_t0", [10]), 7);

// table_copy.wast:1437
assert_return(() => call($17, "check_t0", [11]), 5);

// table_copy.wast:1438
assert_return(() => call($17, "check_t0", [12]), 2);

// table_copy.wast:1439
assert_return(() => call($17, "check_t0", [13]), 3);

// table_copy.wast:1440
assert_return(() => call($17, "check_t0", [14]), 6);

// table_copy.wast:1441
assert_trap(() => call($17, "check_t0", [15]));

// table_copy.wast:1442
assert_trap(() => call($17, "check_t0", [16]));

// table_copy.wast:1443
assert_trap(() => call($17, "check_t0", [17]));

// table_copy.wast:1444
assert_trap(() => call($17, "check_t0", [18]));

// table_copy.wast:1445
assert_trap(() => call($17, "check_t0", [19]));

// table_copy.wast:1446
assert_trap(() => call($17, "check_t0", [20]));

// table_copy.wast:1447
assert_trap(() => call($17, "check_t0", [21]));

// table_copy.wast:1448
assert_trap(() => call($17, "check_t0", [22]));

// table_copy.wast:1449
assert_trap(() => call($17, "check_t0", [23]));

// table_copy.wast:1450
assert_trap(() => call($17, "check_t0", [24]));

// table_copy.wast:1451
assert_trap(() => call($17, "check_t0", [25]));

// table_copy.wast:1452
assert_trap(() => call($17, "check_t0", [26]));

// table_copy.wast:1453
assert_trap(() => call($17, "check_t0", [27]));

// table_copy.wast:1454
assert_trap(() => call($17, "check_t0", [28]));

// table_copy.wast:1455
assert_trap(() => call($17, "check_t0", [29]));

// table_copy.wast:1456
assert_trap(() => call($17, "check_t1", [0]));

// table_copy.wast:1457
assert_trap(() => call($17, "check_t1", [1]));

// table_copy.wast:1458
assert_trap(() => call($17, "check_t1", [2]));

// table_copy.wast:1459
assert_return(() => call($17, "check_t1", [3]), 1);

// table_copy.wast:1460
assert_return(() => call($17, "check_t1", [4]), 3);

// table_copy.wast:1461
assert_return(() => call($17, "check_t1", [5]), 1);

// table_copy.wast:1462
assert_return(() => call($17, "check_t1", [6]), 4);

// table_copy.wast:1463
assert_trap(() => call($17, "check_t1", [7]));

// table_copy.wast:1464
assert_trap(() => call($17, "check_t1", [8]));

// table_copy.wast:1465
assert_trap(() => call($17, "check_t1", [9]));

// table_copy.wast:1466
assert_trap(() => call($17, "check_t1", [10]));

// table_copy.wast:1467
assert_return(() => call($17, "check_t1", [11]), 6);

// table_copy.wast:1468
assert_return(() => call($17, "check_t1", [12]), 3);

// table_copy.wast:1469
assert_return(() => call($17, "check_t1", [13]), 2);

// table_copy.wast:1470
assert_return(() => call($17, "check_t1", [14]), 5);

// table_copy.wast:1471
assert_return(() => call($17, "check_t1", [15]), 7);

// table_copy.wast:1472
assert_trap(() => call($17, "check_t1", [16]));

// table_copy.wast:1473
assert_trap(() => call($17, "check_t1", [17]));

// table_copy.wast:1474
assert_trap(() => call($17, "check_t1", [18]));

// table_copy.wast:1475
assert_trap(() => call($17, "check_t1", [19]));

// table_copy.wast:1476
assert_trap(() => call($17, "check_t1", [20]));

// table_copy.wast:1477
assert_trap(() => call($17, "check_t1", [21]));

// table_copy.wast:1478
assert_trap(() => call($17, "check_t1", [22]));

// table_copy.wast:1479
assert_trap(() => call($17, "check_t1", [23]));

// table_copy.wast:1480
assert_trap(() => call($17, "check_t1", [24]));

// table_copy.wast:1481
assert_trap(() => call($17, "check_t1", [25]));

// table_copy.wast:1482
assert_trap(() => call($17, "check_t1", [26]));

// table_copy.wast:1483
assert_trap(() => call($17, "check_t1", [27]));

// table_copy.wast:1484
assert_trap(() => call($17, "check_t1", [28]));

// table_copy.wast:1485
assert_trap(() => call($17, "check_t1", [29]));

// table_copy.wast:1487
let $18 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x02\x01\x41\x02\x0b\x00\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x02\x01\x41\x0c\x0b\x00\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x00\x41\x03\x0b\x04\x01\x03\x01\x04\x00\x41\x0b\x0b\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0c\x41\x0a\x41\x07\xfc\x0e\x01\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b");

// table_copy.wast:1517
run(() => call($18, "test", []));

// table_copy.wast:1518
assert_trap(() => call($18, "check_t0", [0]));

// table_copy.wast:1519
assert_trap(() => call($18, "check_t0", [1]));

// table_copy.wast:1520
assert_return(() => call($18, "check_t0", [2]), 3);

// table_copy.wast:1521
assert_return(() => call($18, "check_t0", [3]), 1);

// table_copy.wast:1522
assert_return(() => call($18, "check_t0", [4]), 4);

// table_copy.wast:1523
assert_return(() => call($18, "check_t0", [5]), 1);

// table_copy.wast:1524
assert_trap(() => call($18, "check_t0", [6]));

// table_copy.wast:1525
assert_trap(() => call($18, "check_t0", [7]));

// table_copy.wast:1526
assert_trap(() => call($18, "check_t0", [8]));

// table_copy.wast:1527
assert_trap(() => call($18, "check_t0", [9]));

// table_copy.wast:1528
assert_trap(() => call($18, "check_t0", [10]));

// table_copy.wast:1529
assert_trap(() => call($18, "check_t0", [11]));

// table_copy.wast:1530
assert_trap(() => call($18, "check_t0", [12]));

// table_copy.wast:1531
assert_trap(() => call($18, "check_t0", [13]));

// table_copy.wast:1532
assert_return(() => call($18, "check_t0", [14]), 7);

// table_copy.wast:1533
assert_return(() => call($18, "check_t0", [15]), 5);

// table_copy.wast:1534
assert_return(() => call($18, "check_t0", [16]), 2);

// table_copy.wast:1535
assert_return(() => call($18, "check_t0", [17]), 3);

// table_copy.wast:1536
assert_return(() => call($18, "check_t0", [18]), 6);

// table_copy.wast:1537
assert_trap(() => call($18, "check_t0", [19]));

// table_copy.wast:1538
assert_trap(() => call($18, "check_t0", [20]));

// table_copy.wast:1539
assert_trap(() => call($18, "check_t0", [21]));

// table_copy.wast:1540
assert_trap(() => call($18, "check_t0", [22]));

// table_copy.wast:1541
assert_trap(() => call($18, "check_t0", [23]));

// table_copy.wast:1542
assert_trap(() => call($18, "check_t0", [24]));

// table_copy.wast:1543
assert_trap(() => call($18, "check_t0", [25]));

// table_copy.wast:1544
assert_trap(() => call($18, "check_t0", [26]));

// table_copy.wast:1545
assert_trap(() => call($18, "check_t0", [27]));

// table_copy.wast:1546
assert_trap(() => call($18, "check_t0", [28]));

// table_copy.wast:1547
assert_trap(() => call($18, "check_t0", [29]));

// table_copy.wast:1548
assert_trap(() => call($18, "check_t1", [0]));

// table_copy.wast:1549
assert_trap(() => call($18, "check_t1", [1]));

// table_copy.wast:1550
assert_trap(() => call($18, "check_t1", [2]));

// table_copy.wast:1551
assert_return(() => call($18, "check_t1", [3]), 1);

// table_copy.wast:1552
assert_return(() => call($18, "check_t1", [4]), 3);

// table_copy.wast:1553
assert_return(() => call($18, "check_t1", [5]), 1);

// table_copy.wast:1554
assert_return(() => call($18, "check_t1", [6]), 4);

// table_copy.wast:1555
assert_trap(() => call($18, "check_t1", [7]));

// table_copy.wast:1556
assert_trap(() => call($18, "check_t1", [8]));

// table_copy.wast:1557
assert_trap(() => call($18, "check_t1", [9]));

// table_copy.wast:1558
assert_trap(() => call($18, "check_t1", [10]));

// table_copy.wast:1559
assert_return(() => call($18, "check_t1", [11]), 6);

// table_copy.wast:1560
assert_return(() => call($18, "check_t1", [12]), 3);

// table_copy.wast:1561
assert_return(() => call($18, "check_t1", [13]), 2);

// table_copy.wast:1562
assert_return(() => call($18, "check_t1", [14]), 5);

// table_copy.wast:1563
assert_return(() => call($18, "check_t1", [15]), 7);

// table_copy.wast:1564
assert_trap(() => call($18, "check_t1", [16]));

// table_copy.wast:1565
assert_trap(() => call($18, "check_t1", [17]));

// table_copy.wast:1566
assert_trap(() => call($18, "check_t1", [18]));

// table_copy.wast:1567
assert_trap(() => call($18, "check_t1", [19]));

// table_copy.wast:1568
assert_trap(() => call($18, "check_t1", [20]));

// table_copy.wast:1569
assert_trap(() => call($18, "check_t1", [21]));

// table_copy.wast:1570
assert_trap(() => call($18, "check_t1", [22]));

// table_copy.wast:1571
assert_trap(() => call($18, "check_t1", [23]));

// table_copy.wast:1572
assert_trap(() => call($18, "check_t1", [24]));

// table_copy.wast:1573
assert_trap(() => call($18, "check_t1", [25]));

// table_copy.wast:1574
assert_trap(() => call($18, "check_t1", [26]));

// table_copy.wast:1575
assert_trap(() => call($18, "check_t1", [27]));

// table_copy.wast:1576
assert_trap(() => call($18, "check_t1", [28]));

// table_copy.wast:1577
assert_trap(() => call($18, "check_t1", [29]));

// table_copy.wast:1579
let $19 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8d\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7f\x01\x7f\x02\xa9\x80\x80\x80\x00\x05\x01\x61\x03\x65\x66\x30\x00\x00\x01\x61\x03\x65\x66\x31\x00\x00\x01\x61\x03\x65\x66\x32\x00\x00\x01\x61\x03\x65\x66\x33\x00\x00\x01\x61\x03\x65\x66\x34\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x00\x01\x02\x02\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x9e\x80\x80\x80\x00\x03\x04\x74\x65\x73\x74\x00\x0a\x08\x63\x68\x65\x63\x6b\x5f\x74\x30\x00\x0b\x08\x63\x68\x65\x63\x6b\x5f\x74\x31\x00\x0c\x09\xba\x80\x80\x80\x00\x06\x02\x01\x41\x02\x0b\x00\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x02\x01\x41\x0c\x0b\x00\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x00\x41\x03\x0b\x04\x01\x03\x01\x04\x00\x41\x0b\x0b\x05\x06\x03\x02\x05\x07\x0a\xd7\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0a\x41\x00\x41\x14\xfc\x0e\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b");

// table_copy.wast:1609
run(() => call($19, "test", []));

// table_copy.wast:1610
assert_trap(() => call($19, "check_t0", [0]));

// table_copy.wast:1611
assert_trap(() => call($19, "check_t0", [1]));

// table_copy.wast:1612
assert_return(() => call($19, "check_t0", [2]), 3);

// table_copy.wast:1613
assert_return(() => call($19, "check_t0", [3]), 1);

// table_copy.wast:1614
assert_return(() => call($19, "check_t0", [4]), 4);

// table_copy.wast:1615
assert_return(() => call($19, "check_t0", [5]), 1);

// table_copy.wast:1616
assert_trap(() => call($19, "check_t0", [6]));

// table_copy.wast:1617
assert_trap(() => call($19, "check_t0", [7]));

// table_copy.wast:1618
assert_trap(() => call($19, "check_t0", [8]));

// table_copy.wast:1619
assert_trap(() => call($19, "check_t0", [9]));

// table_copy.wast:1620
assert_trap(() => call($19, "check_t0", [10]));

// table_copy.wast:1621
assert_trap(() => call($19, "check_t0", [11]));

// table_copy.wast:1622
assert_return(() => call($19, "check_t0", [12]), 7);

// table_copy.wast:1623
assert_return(() => call($19, "check_t0", [13]), 5);

// table_copy.wast:1624
assert_return(() => call($19, "check_t0", [14]), 2);

// table_copy.wast:1625
assert_return(() => call($19, "check_t0", [15]), 3);

// table_copy.wast:1626
assert_return(() => call($19, "check_t0", [16]), 6);

// table_copy.wast:1627
assert_trap(() => call($19, "check_t0", [17]));

// table_copy.wast:1628
assert_trap(() => call($19, "check_t0", [18]));

// table_copy.wast:1629
assert_trap(() => call($19, "check_t0", [19]));

// table_copy.wast:1630
assert_trap(() => call($19, "check_t0", [20]));

// table_copy.wast:1631
assert_trap(() => call($19, "check_t0", [21]));

// table_copy.wast:1632
assert_trap(() => call($19, "check_t0", [22]));

// table_copy.wast:1633
assert_trap(() => call($19, "check_t0", [23]));

// table_copy.wast:1634
assert_trap(() => call($19, "check_t0", [24]));

// table_copy.wast:1635
assert_trap(() => call($19, "check_t0", [25]));

// table_copy.wast:1636
assert_trap(() => call($19, "check_t0", [26]));

// table_copy.wast:1637
assert_trap(() => call($19, "check_t0", [27]));

// table_copy.wast:1638
assert_trap(() => call($19, "check_t0", [28]));

// table_copy.wast:1639
assert_trap(() => call($19, "check_t0", [29]));

// table_copy.wast:1640
assert_trap(() => call($19, "check_t1", [0]));

// table_copy.wast:1641
assert_trap(() => call($19, "check_t1", [1]));

// table_copy.wast:1642
assert_trap(() => call($19, "check_t1", [2]));

// table_copy.wast:1643
assert_return(() => call($19, "check_t1", [3]), 1);

// table_copy.wast:1644
assert_return(() => call($19, "check_t1", [4]), 3);

// table_copy.wast:1645
assert_return(() => call($19, "check_t1", [5]), 1);

// table_copy.wast:1646
assert_return(() => call($19, "check_t1", [6]), 4);

// table_copy.wast:1647
assert_trap(() => call($19, "check_t1", [7]));

// table_copy.wast:1648
assert_trap(() => call($19, "check_t1", [8]));

// table_copy.wast:1649
assert_trap(() => call($19, "check_t1", [9]));

// table_copy.wast:1650
assert_trap(() => call($19, "check_t1", [10]));

// table_copy.wast:1651
assert_trap(() => call($19, "check_t1", [11]));

// table_copy.wast:1652
assert_return(() => call($19, "check_t1", [12]), 3);

// table_copy.wast:1653
assert_return(() => call($19, "check_t1", [13]), 1);

// table_copy.wast:1654
assert_return(() => call($19, "check_t1", [14]), 4);

// table_copy.wast:1655
assert_return(() => call($19, "check_t1", [15]), 1);

// table_copy.wast:1656
assert_trap(() => call($19, "check_t1", [16]));

// table_copy.wast:1657
assert_trap(() => call($19, "check_t1", [17]));

// table_copy.wast:1658
assert_trap(() => call($19, "check_t1", [18]));

// table_copy.wast:1659
assert_trap(() => call($19, "check_t1", [19]));

// table_copy.wast:1660
assert_trap(() => call($19, "check_t1", [20]));

// table_copy.wast:1661
assert_trap(() => call($19, "check_t1", [21]));

// table_copy.wast:1662
assert_return(() => call($19, "check_t1", [22]), 7);

// table_copy.wast:1663
assert_return(() => call($19, "check_t1", [23]), 5);

// table_copy.wast:1664
assert_return(() => call($19, "check_t1", [24]), 2);

// table_copy.wast:1665
assert_return(() => call($19, "check_t1", [25]), 3);

// table_copy.wast:1666
assert_return(() => call($19, "check_t1", [26]), 6);

// table_copy.wast:1667
assert_trap(() => call($19, "check_t1", [27]));

// table_copy.wast:1668
assert_trap(() => call($19, "check_t1", [28]));

// table_copy.wast:1669
assert_trap(() => call($19, "check_t1", [29]));

// table_copy.wast:1671
let $20 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1c\x41\x01\x41\x03\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1694
assert_trap(() => call($20, "test", []));

// table_copy.wast:1696
let $21 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x7e\x41\x01\x41\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1719
assert_trap(() => call($21, "test", []));

// table_copy.wast:1721
let $22 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x19\x41\x06\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1744
assert_trap(() => call($22, "test", []));

// table_copy.wast:1746
let $23 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x7e\x41\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1769
assert_trap(() => call($23, "test", []));

// table_copy.wast:1771
let $24 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x19\x41\x00\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1794
run(() => call($24, "test", []));

// table_copy.wast:1796
let $25 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1e\x41\x0f\x41\x00\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1819
run(() => call($25, "test", []));

// table_copy.wast:1821
let $26 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1f\x41\x0f\x41\x00\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1844
assert_trap(() => call($26, "test", []));

// table_copy.wast:1846
let $27 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x1e\x41\x00\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1869
run(() => call($27, "test", []));

// table_copy.wast:1871
let $28 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x1f\x41\x00\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1894
assert_trap(() => call($28, "test", []));

// table_copy.wast:1896
let $29 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1e\x41\x1e\x41\x00\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1919
run(() => call($29, "test", []));

// table_copy.wast:1921
let $30 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1f\x41\x1f\x41\x00\xfc\x0e\x00\x00\x0b");

// table_copy.wast:1944
assert_trap(() => call($30, "test", []));

// table_copy.wast:1946
let $31 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1c\x41\x01\x41\x03\xfc\x0e\x01\x00\x0b");

// table_copy.wast:1969
assert_trap(() => call($31, "test", []));

// table_copy.wast:1971
let $32 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x7e\x41\x01\x41\x02\xfc\x0e\x01\x00\x0b");

// table_copy.wast:1994
assert_trap(() => call($32, "test", []));

// table_copy.wast:1996
let $33 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x19\x41\x06\xfc\x0e\x01\x00\x0b");

// table_copy.wast:2019
assert_trap(() => call($33, "test", []));

// table_copy.wast:2021
let $34 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x7e\x41\x02\xfc\x0e\x01\x00\x0b");

// table_copy.wast:2044
assert_trap(() => call($34, "test", []));

// table_copy.wast:2046
let $35 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x19\x41\x00\xfc\x0e\x01\x00\x0b");

// table_copy.wast:2069
run(() => call($35, "test", []));

// table_copy.wast:2071
let $36 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1e\x41\x0f\x41\x00\xfc\x0e\x01\x00\x0b");

// table_copy.wast:2094
run(() => call($36, "test", []));

// table_copy.wast:2096
let $37 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1f\x41\x0f\x41\x00\xfc\x0e\x01\x00\x0b");

// table_copy.wast:2119
assert_trap(() => call($37, "test", []));

// table_copy.wast:2121
let $38 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x1e\x41\x00\xfc\x0e\x01\x00\x0b");

// table_copy.wast:2144
run(() => call($38, "test", []));

// table_copy.wast:2146
let $39 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x0f\x41\x1f\x41\x00\xfc\x0e\x01\x00\x0b");

// table_copy.wast:2169
assert_trap(() => call($39, "test", []));

// table_copy.wast:2171
let $40 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1e\x41\x1e\x41\x00\xfc\x0e\x01\x00\x0b");

// table_copy.wast:2194
run(() => call($40, "test", []));

// table_copy.wast:2196
let $41 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x88\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x00\x00\x03\x8c\x80\x80\x80\x00\x0b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04\x89\x80\x80\x80\x00\x02\x70\x01\x1e\x1e\x70\x01\x1e\x1e\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x0a\x09\xa3\x80\x80\x80\x00\x04\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x01\x00\x04\x02\x07\x01\x08\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06\x01\x00\x05\x05\x09\x02\x07\x06\x0a\xec\x80\x80\x80\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x8c\x80\x80\x80\x00\x00\x41\x1f\x41\x1f\x41\x00\xfc\x0e\x01\x00\x0b");

// table_copy.wast:2219
assert_trap(() => call($41, "test", []));

// table_copy.wast:2221
let $42 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x85\x80\x80\x80\x00\x01\x70\x01\x20\x40\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x8e\x80\x80\x80\x00\x01\x00\x41\x00\x0b\x08\x00\x01\x02\x03\x04\x05\x06\x07\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2247
assert_trap(() => call($42, "run", [24, 0, 16]));

// table_copy.wast:2249
assert_return(() => call($42, "test", [0]), 0);

// table_copy.wast:2250
assert_return(() => call($42, "test", [1]), 1);

// table_copy.wast:2251
assert_return(() => call($42, "test", [2]), 2);

// table_copy.wast:2252
assert_return(() => call($42, "test", [3]), 3);

// table_copy.wast:2253
assert_return(() => call($42, "test", [4]), 4);

// table_copy.wast:2254
assert_return(() => call($42, "test", [5]), 5);

// table_copy.wast:2255
assert_return(() => call($42, "test", [6]), 6);

// table_copy.wast:2256
assert_return(() => call($42, "test", [7]), 7);

// table_copy.wast:2257
assert_trap(() => call($42, "test", [8]));

// table_copy.wast:2258
assert_trap(() => call($42, "test", [9]));

// table_copy.wast:2259
assert_trap(() => call($42, "test", [10]));

// table_copy.wast:2260
assert_trap(() => call($42, "test", [11]));

// table_copy.wast:2261
assert_trap(() => call($42, "test", [12]));

// table_copy.wast:2262
assert_trap(() => call($42, "test", [13]));

// table_copy.wast:2263
assert_trap(() => call($42, "test", [14]));

// table_copy.wast:2264
assert_trap(() => call($42, "test", [15]));

// table_copy.wast:2265
assert_trap(() => call($42, "test", [16]));

// table_copy.wast:2266
assert_trap(() => call($42, "test", [17]));

// table_copy.wast:2267
assert_trap(() => call($42, "test", [18]));

// table_copy.wast:2268
assert_trap(() => call($42, "test", [19]));

// table_copy.wast:2269
assert_trap(() => call($42, "test", [20]));

// table_copy.wast:2270
assert_trap(() => call($42, "test", [21]));

// table_copy.wast:2271
assert_trap(() => call($42, "test", [22]));

// table_copy.wast:2272
assert_trap(() => call($42, "test", [23]));

// table_copy.wast:2273
assert_trap(() => call($42, "test", [24]));

// table_copy.wast:2274
assert_trap(() => call($42, "test", [25]));

// table_copy.wast:2275
assert_trap(() => call($42, "test", [26]));

// table_copy.wast:2276
assert_trap(() => call($42, "test", [27]));

// table_copy.wast:2277
assert_trap(() => call($42, "test", [28]));

// table_copy.wast:2278
assert_trap(() => call($42, "test", [29]));

// table_copy.wast:2279
assert_trap(() => call($42, "test", [30]));

// table_copy.wast:2280
assert_trap(() => call($42, "test", [31]));

// table_copy.wast:2282
let $43 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x85\x80\x80\x80\x00\x01\x70\x01\x20\x40\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x8f\x80\x80\x80\x00\x01\x00\x41\x00\x0b\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2308
assert_trap(() => call($43, "run", [23, 0, 15]));

// table_copy.wast:2310
assert_return(() => call($43, "test", [0]), 0);

// table_copy.wast:2311
assert_return(() => call($43, "test", [1]), 1);

// table_copy.wast:2312
assert_return(() => call($43, "test", [2]), 2);

// table_copy.wast:2313
assert_return(() => call($43, "test", [3]), 3);

// table_copy.wast:2314
assert_return(() => call($43, "test", [4]), 4);

// table_copy.wast:2315
assert_return(() => call($43, "test", [5]), 5);

// table_copy.wast:2316
assert_return(() => call($43, "test", [6]), 6);

// table_copy.wast:2317
assert_return(() => call($43, "test", [7]), 7);

// table_copy.wast:2318
assert_return(() => call($43, "test", [8]), 8);

// table_copy.wast:2319
assert_trap(() => call($43, "test", [9]));

// table_copy.wast:2320
assert_trap(() => call($43, "test", [10]));

// table_copy.wast:2321
assert_trap(() => call($43, "test", [11]));

// table_copy.wast:2322
assert_trap(() => call($43, "test", [12]));

// table_copy.wast:2323
assert_trap(() => call($43, "test", [13]));

// table_copy.wast:2324
assert_trap(() => call($43, "test", [14]));

// table_copy.wast:2325
assert_trap(() => call($43, "test", [15]));

// table_copy.wast:2326
assert_trap(() => call($43, "test", [16]));

// table_copy.wast:2327
assert_trap(() => call($43, "test", [17]));

// table_copy.wast:2328
assert_trap(() => call($43, "test", [18]));

// table_copy.wast:2329
assert_trap(() => call($43, "test", [19]));

// table_copy.wast:2330
assert_trap(() => call($43, "test", [20]));

// table_copy.wast:2331
assert_trap(() => call($43, "test", [21]));

// table_copy.wast:2332
assert_trap(() => call($43, "test", [22]));

// table_copy.wast:2333
assert_trap(() => call($43, "test", [23]));

// table_copy.wast:2334
assert_trap(() => call($43, "test", [24]));

// table_copy.wast:2335
assert_trap(() => call($43, "test", [25]));

// table_copy.wast:2336
assert_trap(() => call($43, "test", [26]));

// table_copy.wast:2337
assert_trap(() => call($43, "test", [27]));

// table_copy.wast:2338
assert_trap(() => call($43, "test", [28]));

// table_copy.wast:2339
assert_trap(() => call($43, "test", [29]));

// table_copy.wast:2340
assert_trap(() => call($43, "test", [30]));

// table_copy.wast:2341
assert_trap(() => call($43, "test", [31]));

// table_copy.wast:2343
let $44 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x85\x80\x80\x80\x00\x01\x70\x01\x20\x40\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x8e\x80\x80\x80\x00\x01\x00\x41\x18\x0b\x08\x00\x01\x02\x03\x04\x05\x06\x07\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2369
assert_trap(() => call($44, "run", [0, 24, 16]));

// table_copy.wast:2371
assert_trap(() => call($44, "test", [0]));

// table_copy.wast:2372
assert_trap(() => call($44, "test", [1]));

// table_copy.wast:2373
assert_trap(() => call($44, "test", [2]));

// table_copy.wast:2374
assert_trap(() => call($44, "test", [3]));

// table_copy.wast:2375
assert_trap(() => call($44, "test", [4]));

// table_copy.wast:2376
assert_trap(() => call($44, "test", [5]));

// table_copy.wast:2377
assert_trap(() => call($44, "test", [6]));

// table_copy.wast:2378
assert_trap(() => call($44, "test", [7]));

// table_copy.wast:2379
assert_trap(() => call($44, "test", [8]));

// table_copy.wast:2380
assert_trap(() => call($44, "test", [9]));

// table_copy.wast:2381
assert_trap(() => call($44, "test", [10]));

// table_copy.wast:2382
assert_trap(() => call($44, "test", [11]));

// table_copy.wast:2383
assert_trap(() => call($44, "test", [12]));

// table_copy.wast:2384
assert_trap(() => call($44, "test", [13]));

// table_copy.wast:2385
assert_trap(() => call($44, "test", [14]));

// table_copy.wast:2386
assert_trap(() => call($44, "test", [15]));

// table_copy.wast:2387
assert_trap(() => call($44, "test", [16]));

// table_copy.wast:2388
assert_trap(() => call($44, "test", [17]));

// table_copy.wast:2389
assert_trap(() => call($44, "test", [18]));

// table_copy.wast:2390
assert_trap(() => call($44, "test", [19]));

// table_copy.wast:2391
assert_trap(() => call($44, "test", [20]));

// table_copy.wast:2392
assert_trap(() => call($44, "test", [21]));

// table_copy.wast:2393
assert_trap(() => call($44, "test", [22]));

// table_copy.wast:2394
assert_trap(() => call($44, "test", [23]));

// table_copy.wast:2395
assert_return(() => call($44, "test", [24]), 0);

// table_copy.wast:2396
assert_return(() => call($44, "test", [25]), 1);

// table_copy.wast:2397
assert_return(() => call($44, "test", [26]), 2);

// table_copy.wast:2398
assert_return(() => call($44, "test", [27]), 3);

// table_copy.wast:2399
assert_return(() => call($44, "test", [28]), 4);

// table_copy.wast:2400
assert_return(() => call($44, "test", [29]), 5);

// table_copy.wast:2401
assert_return(() => call($44, "test", [30]), 6);

// table_copy.wast:2402
assert_return(() => call($44, "test", [31]), 7);

// table_copy.wast:2404
let $45 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x85\x80\x80\x80\x00\x01\x70\x01\x20\x40\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x8f\x80\x80\x80\x00\x01\x00\x41\x17\x0b\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2430
assert_trap(() => call($45, "run", [0, 23, 15]));

// table_copy.wast:2432
assert_trap(() => call($45, "test", [0]));

// table_copy.wast:2433
assert_trap(() => call($45, "test", [1]));

// table_copy.wast:2434
assert_trap(() => call($45, "test", [2]));

// table_copy.wast:2435
assert_trap(() => call($45, "test", [3]));

// table_copy.wast:2436
assert_trap(() => call($45, "test", [4]));

// table_copy.wast:2437
assert_trap(() => call($45, "test", [5]));

// table_copy.wast:2438
assert_trap(() => call($45, "test", [6]));

// table_copy.wast:2439
assert_trap(() => call($45, "test", [7]));

// table_copy.wast:2440
assert_trap(() => call($45, "test", [8]));

// table_copy.wast:2441
assert_trap(() => call($45, "test", [9]));

// table_copy.wast:2442
assert_trap(() => call($45, "test", [10]));

// table_copy.wast:2443
assert_trap(() => call($45, "test", [11]));

// table_copy.wast:2444
assert_trap(() => call($45, "test", [12]));

// table_copy.wast:2445
assert_trap(() => call($45, "test", [13]));

// table_copy.wast:2446
assert_trap(() => call($45, "test", [14]));

// table_copy.wast:2447
assert_trap(() => call($45, "test", [15]));

// table_copy.wast:2448
assert_trap(() => call($45, "test", [16]));

// table_copy.wast:2449
assert_trap(() => call($45, "test", [17]));

// table_copy.wast:2450
assert_trap(() => call($45, "test", [18]));

// table_copy.wast:2451
assert_trap(() => call($45, "test", [19]));

// table_copy.wast:2452
assert_trap(() => call($45, "test", [20]));

// table_copy.wast:2453
assert_trap(() => call($45, "test", [21]));

// table_copy.wast:2454
assert_trap(() => call($45, "test", [22]));

// table_copy.wast:2455
assert_return(() => call($45, "test", [23]), 0);

// table_copy.wast:2456
assert_return(() => call($45, "test", [24]), 1);

// table_copy.wast:2457
assert_return(() => call($45, "test", [25]), 2);

// table_copy.wast:2458
assert_return(() => call($45, "test", [26]), 3);

// table_copy.wast:2459
assert_return(() => call($45, "test", [27]), 4);

// table_copy.wast:2460
assert_return(() => call($45, "test", [28]), 5);

// table_copy.wast:2461
assert_return(() => call($45, "test", [29]), 6);

// table_copy.wast:2462
assert_return(() => call($45, "test", [30]), 7);

// table_copy.wast:2463
assert_return(() => call($45, "test", [31]), 8);

// table_copy.wast:2465
let $46 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x85\x80\x80\x80\x00\x01\x70\x01\x20\x40\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x8e\x80\x80\x80\x00\x01\x00\x41\x0b\x0b\x08\x00\x01\x02\x03\x04\x05\x06\x07\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2491
assert_trap(() => call($46, "run", [24, 11, 16]));

// table_copy.wast:2493
assert_trap(() => call($46, "test", [0]));

// table_copy.wast:2494
assert_trap(() => call($46, "test", [1]));

// table_copy.wast:2495
assert_trap(() => call($46, "test", [2]));

// table_copy.wast:2496
assert_trap(() => call($46, "test", [3]));

// table_copy.wast:2497
assert_trap(() => call($46, "test", [4]));

// table_copy.wast:2498
assert_trap(() => call($46, "test", [5]));

// table_copy.wast:2499
assert_trap(() => call($46, "test", [6]));

// table_copy.wast:2500
assert_trap(() => call($46, "test", [7]));

// table_copy.wast:2501
assert_trap(() => call($46, "test", [8]));

// table_copy.wast:2502
assert_trap(() => call($46, "test", [9]));

// table_copy.wast:2503
assert_trap(() => call($46, "test", [10]));

// table_copy.wast:2504
assert_return(() => call($46, "test", [11]), 0);

// table_copy.wast:2505
assert_return(() => call($46, "test", [12]), 1);

// table_copy.wast:2506
assert_return(() => call($46, "test", [13]), 2);

// table_copy.wast:2507
assert_return(() => call($46, "test", [14]), 3);

// table_copy.wast:2508
assert_return(() => call($46, "test", [15]), 4);

// table_copy.wast:2509
assert_return(() => call($46, "test", [16]), 5);

// table_copy.wast:2510
assert_return(() => call($46, "test", [17]), 6);

// table_copy.wast:2511
assert_return(() => call($46, "test", [18]), 7);

// table_copy.wast:2512
assert_trap(() => call($46, "test", [19]));

// table_copy.wast:2513
assert_trap(() => call($46, "test", [20]));

// table_copy.wast:2514
assert_trap(() => call($46, "test", [21]));

// table_copy.wast:2515
assert_trap(() => call($46, "test", [22]));

// table_copy.wast:2516
assert_trap(() => call($46, "test", [23]));

// table_copy.wast:2517
assert_trap(() => call($46, "test", [24]));

// table_copy.wast:2518
assert_trap(() => call($46, "test", [25]));

// table_copy.wast:2519
assert_trap(() => call($46, "test", [26]));

// table_copy.wast:2520
assert_trap(() => call($46, "test", [27]));

// table_copy.wast:2521
assert_trap(() => call($46, "test", [28]));

// table_copy.wast:2522
assert_trap(() => call($46, "test", [29]));

// table_copy.wast:2523
assert_trap(() => call($46, "test", [30]));

// table_copy.wast:2524
assert_trap(() => call($46, "test", [31]));

// table_copy.wast:2526
let $47 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x85\x80\x80\x80\x00\x01\x70\x01\x20\x40\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x8e\x80\x80\x80\x00\x01\x00\x41\x18\x0b\x08\x00\x01\x02\x03\x04\x05\x06\x07\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2552
assert_trap(() => call($47, "run", [11, 24, 16]));

// table_copy.wast:2554
assert_trap(() => call($47, "test", [0]));

// table_copy.wast:2555
assert_trap(() => call($47, "test", [1]));

// table_copy.wast:2556
assert_trap(() => call($47, "test", [2]));

// table_copy.wast:2557
assert_trap(() => call($47, "test", [3]));

// table_copy.wast:2558
assert_trap(() => call($47, "test", [4]));

// table_copy.wast:2559
assert_trap(() => call($47, "test", [5]));

// table_copy.wast:2560
assert_trap(() => call($47, "test", [6]));

// table_copy.wast:2561
assert_trap(() => call($47, "test", [7]));

// table_copy.wast:2562
assert_trap(() => call($47, "test", [8]));

// table_copy.wast:2563
assert_trap(() => call($47, "test", [9]));

// table_copy.wast:2564
assert_trap(() => call($47, "test", [10]));

// table_copy.wast:2565
assert_trap(() => call($47, "test", [11]));

// table_copy.wast:2566
assert_trap(() => call($47, "test", [12]));

// table_copy.wast:2567
assert_trap(() => call($47, "test", [13]));

// table_copy.wast:2568
assert_trap(() => call($47, "test", [14]));

// table_copy.wast:2569
assert_trap(() => call($47, "test", [15]));

// table_copy.wast:2570
assert_trap(() => call($47, "test", [16]));

// table_copy.wast:2571
assert_trap(() => call($47, "test", [17]));

// table_copy.wast:2572
assert_trap(() => call($47, "test", [18]));

// table_copy.wast:2573
assert_trap(() => call($47, "test", [19]));

// table_copy.wast:2574
assert_trap(() => call($47, "test", [20]));

// table_copy.wast:2575
assert_trap(() => call($47, "test", [21]));

// table_copy.wast:2576
assert_trap(() => call($47, "test", [22]));

// table_copy.wast:2577
assert_trap(() => call($47, "test", [23]));

// table_copy.wast:2578
assert_return(() => call($47, "test", [24]), 0);

// table_copy.wast:2579
assert_return(() => call($47, "test", [25]), 1);

// table_copy.wast:2580
assert_return(() => call($47, "test", [26]), 2);

// table_copy.wast:2581
assert_return(() => call($47, "test", [27]), 3);

// table_copy.wast:2582
assert_return(() => call($47, "test", [28]), 4);

// table_copy.wast:2583
assert_return(() => call($47, "test", [29]), 5);

// table_copy.wast:2584
assert_return(() => call($47, "test", [30]), 6);

// table_copy.wast:2585
assert_return(() => call($47, "test", [31]), 7);

// table_copy.wast:2587
let $48 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x85\x80\x80\x80\x00\x01\x70\x01\x20\x40\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x8e\x80\x80\x80\x00\x01\x00\x41\x15\x0b\x08\x00\x01\x02\x03\x04\x05\x06\x07\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2613
assert_trap(() => call($48, "run", [24, 21, 16]));

// table_copy.wast:2615
assert_trap(() => call($48, "test", [0]));

// table_copy.wast:2616
assert_trap(() => call($48, "test", [1]));

// table_copy.wast:2617
assert_trap(() => call($48, "test", [2]));

// table_copy.wast:2618
assert_trap(() => call($48, "test", [3]));

// table_copy.wast:2619
assert_trap(() => call($48, "test", [4]));

// table_copy.wast:2620
assert_trap(() => call($48, "test", [5]));

// table_copy.wast:2621
assert_trap(() => call($48, "test", [6]));

// table_copy.wast:2622
assert_trap(() => call($48, "test", [7]));

// table_copy.wast:2623
assert_trap(() => call($48, "test", [8]));

// table_copy.wast:2624
assert_trap(() => call($48, "test", [9]));

// table_copy.wast:2625
assert_trap(() => call($48, "test", [10]));

// table_copy.wast:2626
assert_trap(() => call($48, "test", [11]));

// table_copy.wast:2627
assert_trap(() => call($48, "test", [12]));

// table_copy.wast:2628
assert_trap(() => call($48, "test", [13]));

// table_copy.wast:2629
assert_trap(() => call($48, "test", [14]));

// table_copy.wast:2630
assert_trap(() => call($48, "test", [15]));

// table_copy.wast:2631
assert_trap(() => call($48, "test", [16]));

// table_copy.wast:2632
assert_trap(() => call($48, "test", [17]));

// table_copy.wast:2633
assert_trap(() => call($48, "test", [18]));

// table_copy.wast:2634
assert_trap(() => call($48, "test", [19]));

// table_copy.wast:2635
assert_trap(() => call($48, "test", [20]));

// table_copy.wast:2636
assert_return(() => call($48, "test", [21]), 0);

// table_copy.wast:2637
assert_return(() => call($48, "test", [22]), 1);

// table_copy.wast:2638
assert_return(() => call($48, "test", [23]), 2);

// table_copy.wast:2639
assert_return(() => call($48, "test", [24]), 3);

// table_copy.wast:2640
assert_return(() => call($48, "test", [25]), 4);

// table_copy.wast:2641
assert_return(() => call($48, "test", [26]), 5);

// table_copy.wast:2642
assert_return(() => call($48, "test", [27]), 6);

// table_copy.wast:2643
assert_return(() => call($48, "test", [28]), 7);

// table_copy.wast:2644
assert_trap(() => call($48, "test", [29]));

// table_copy.wast:2645
assert_trap(() => call($48, "test", [30]));

// table_copy.wast:2646
assert_trap(() => call($48, "test", [31]));

// table_copy.wast:2648
let $49 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x85\x80\x80\x80\x00\x01\x70\x01\x20\x40\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x8e\x80\x80\x80\x00\x01\x00\x41\x18\x0b\x08\x00\x01\x02\x03\x04\x05\x06\x07\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2674
assert_trap(() => call($49, "run", [21, 24, 16]));

// table_copy.wast:2676
assert_trap(() => call($49, "test", [0]));

// table_copy.wast:2677
assert_trap(() => call($49, "test", [1]));

// table_copy.wast:2678
assert_trap(() => call($49, "test", [2]));

// table_copy.wast:2679
assert_trap(() => call($49, "test", [3]));

// table_copy.wast:2680
assert_trap(() => call($49, "test", [4]));

// table_copy.wast:2681
assert_trap(() => call($49, "test", [5]));

// table_copy.wast:2682
assert_trap(() => call($49, "test", [6]));

// table_copy.wast:2683
assert_trap(() => call($49, "test", [7]));

// table_copy.wast:2684
assert_trap(() => call($49, "test", [8]));

// table_copy.wast:2685
assert_trap(() => call($49, "test", [9]));

// table_copy.wast:2686
assert_trap(() => call($49, "test", [10]));

// table_copy.wast:2687
assert_trap(() => call($49, "test", [11]));

// table_copy.wast:2688
assert_trap(() => call($49, "test", [12]));

// table_copy.wast:2689
assert_trap(() => call($49, "test", [13]));

// table_copy.wast:2690
assert_trap(() => call($49, "test", [14]));

// table_copy.wast:2691
assert_trap(() => call($49, "test", [15]));

// table_copy.wast:2692
assert_trap(() => call($49, "test", [16]));

// table_copy.wast:2693
assert_trap(() => call($49, "test", [17]));

// table_copy.wast:2694
assert_trap(() => call($49, "test", [18]));

// table_copy.wast:2695
assert_trap(() => call($49, "test", [19]));

// table_copy.wast:2696
assert_trap(() => call($49, "test", [20]));

// table_copy.wast:2697
assert_trap(() => call($49, "test", [21]));

// table_copy.wast:2698
assert_trap(() => call($49, "test", [22]));

// table_copy.wast:2699
assert_trap(() => call($49, "test", [23]));

// table_copy.wast:2700
assert_return(() => call($49, "test", [24]), 0);

// table_copy.wast:2701
assert_return(() => call($49, "test", [25]), 1);

// table_copy.wast:2702
assert_return(() => call($49, "test", [26]), 2);

// table_copy.wast:2703
assert_return(() => call($49, "test", [27]), 3);

// table_copy.wast:2704
assert_return(() => call($49, "test", [28]), 4);

// table_copy.wast:2705
assert_return(() => call($49, "test", [29]), 5);

// table_copy.wast:2706
assert_return(() => call($49, "test", [30]), 6);

// table_copy.wast:2707
assert_return(() => call($49, "test", [31]), 7);

// table_copy.wast:2709
let $50 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x85\x80\x80\x80\x00\x01\x70\x01\x20\x40\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x91\x80\x80\x80\x00\x01\x00\x41\x15\x0b\x0b\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2735
assert_trap(() => call($50, "run", [21, 21, 16]));

// table_copy.wast:2737
assert_trap(() => call($50, "test", [0]));

// table_copy.wast:2738
assert_trap(() => call($50, "test", [1]));

// table_copy.wast:2739
assert_trap(() => call($50, "test", [2]));

// table_copy.wast:2740
assert_trap(() => call($50, "test", [3]));

// table_copy.wast:2741
assert_trap(() => call($50, "test", [4]));

// table_copy.wast:2742
assert_trap(() => call($50, "test", [5]));

// table_copy.wast:2743
assert_trap(() => call($50, "test", [6]));

// table_copy.wast:2744
assert_trap(() => call($50, "test", [7]));

// table_copy.wast:2745
assert_trap(() => call($50, "test", [8]));

// table_copy.wast:2746
assert_trap(() => call($50, "test", [9]));

// table_copy.wast:2747
assert_trap(() => call($50, "test", [10]));

// table_copy.wast:2748
assert_trap(() => call($50, "test", [11]));

// table_copy.wast:2749
assert_trap(() => call($50, "test", [12]));

// table_copy.wast:2750
assert_trap(() => call($50, "test", [13]));

// table_copy.wast:2751
assert_trap(() => call($50, "test", [14]));

// table_copy.wast:2752
assert_trap(() => call($50, "test", [15]));

// table_copy.wast:2753
assert_trap(() => call($50, "test", [16]));

// table_copy.wast:2754
assert_trap(() => call($50, "test", [17]));

// table_copy.wast:2755
assert_trap(() => call($50, "test", [18]));

// table_copy.wast:2756
assert_trap(() => call($50, "test", [19]));

// table_copy.wast:2757
assert_trap(() => call($50, "test", [20]));

// table_copy.wast:2758
assert_return(() => call($50, "test", [21]), 0);

// table_copy.wast:2759
assert_return(() => call($50, "test", [22]), 1);

// table_copy.wast:2760
assert_return(() => call($50, "test", [23]), 2);

// table_copy.wast:2761
assert_return(() => call($50, "test", [24]), 3);

// table_copy.wast:2762
assert_return(() => call($50, "test", [25]), 4);

// table_copy.wast:2763
assert_return(() => call($50, "test", [26]), 5);

// table_copy.wast:2764
assert_return(() => call($50, "test", [27]), 6);

// table_copy.wast:2765
assert_return(() => call($50, "test", [28]), 7);

// table_copy.wast:2766
assert_return(() => call($50, "test", [29]), 8);

// table_copy.wast:2767
assert_return(() => call($50, "test", [30]), 9);

// table_copy.wast:2768
assert_return(() => call($50, "test", [31]), 10);

// table_copy.wast:2770
let $51 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x87\x80\x80\x80\x00\x01\x70\x01\x80\x01\x80\x01\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x97\x80\x80\x80\x00\x01\x00\x41\xf0\x00\x0b\x10\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2796
assert_trap(() => call($51, "run", [0, 112, -32]));

// table_copy.wast:2798
assert_trap(() => call($51, "test", [0]));

// table_copy.wast:2799
assert_trap(() => call($51, "test", [1]));

// table_copy.wast:2800
assert_trap(() => call($51, "test", [2]));

// table_copy.wast:2801
assert_trap(() => call($51, "test", [3]));

// table_copy.wast:2802
assert_trap(() => call($51, "test", [4]));

// table_copy.wast:2803
assert_trap(() => call($51, "test", [5]));

// table_copy.wast:2804
assert_trap(() => call($51, "test", [6]));

// table_copy.wast:2805
assert_trap(() => call($51, "test", [7]));

// table_copy.wast:2806
assert_trap(() => call($51, "test", [8]));

// table_copy.wast:2807
assert_trap(() => call($51, "test", [9]));

// table_copy.wast:2808
assert_trap(() => call($51, "test", [10]));

// table_copy.wast:2809
assert_trap(() => call($51, "test", [11]));

// table_copy.wast:2810
assert_trap(() => call($51, "test", [12]));

// table_copy.wast:2811
assert_trap(() => call($51, "test", [13]));

// table_copy.wast:2812
assert_trap(() => call($51, "test", [14]));

// table_copy.wast:2813
assert_trap(() => call($51, "test", [15]));

// table_copy.wast:2814
assert_trap(() => call($51, "test", [16]));

// table_copy.wast:2815
assert_trap(() => call($51, "test", [17]));

// table_copy.wast:2816
assert_trap(() => call($51, "test", [18]));

// table_copy.wast:2817
assert_trap(() => call($51, "test", [19]));

// table_copy.wast:2818
assert_trap(() => call($51, "test", [20]));

// table_copy.wast:2819
assert_trap(() => call($51, "test", [21]));

// table_copy.wast:2820
assert_trap(() => call($51, "test", [22]));

// table_copy.wast:2821
assert_trap(() => call($51, "test", [23]));

// table_copy.wast:2822
assert_trap(() => call($51, "test", [24]));

// table_copy.wast:2823
assert_trap(() => call($51, "test", [25]));

// table_copy.wast:2824
assert_trap(() => call($51, "test", [26]));

// table_copy.wast:2825
assert_trap(() => call($51, "test", [27]));

// table_copy.wast:2826
assert_trap(() => call($51, "test", [28]));

// table_copy.wast:2827
assert_trap(() => call($51, "test", [29]));

// table_copy.wast:2828
assert_trap(() => call($51, "test", [30]));

// table_copy.wast:2829
assert_trap(() => call($51, "test", [31]));

// table_copy.wast:2830
assert_trap(() => call($51, "test", [32]));

// table_copy.wast:2831
assert_trap(() => call($51, "test", [33]));

// table_copy.wast:2832
assert_trap(() => call($51, "test", [34]));

// table_copy.wast:2833
assert_trap(() => call($51, "test", [35]));

// table_copy.wast:2834
assert_trap(() => call($51, "test", [36]));

// table_copy.wast:2835
assert_trap(() => call($51, "test", [37]));

// table_copy.wast:2836
assert_trap(() => call($51, "test", [38]));

// table_copy.wast:2837
assert_trap(() => call($51, "test", [39]));

// table_copy.wast:2838
assert_trap(() => call($51, "test", [40]));

// table_copy.wast:2839
assert_trap(() => call($51, "test", [41]));

// table_copy.wast:2840
assert_trap(() => call($51, "test", [42]));

// table_copy.wast:2841
assert_trap(() => call($51, "test", [43]));

// table_copy.wast:2842
assert_trap(() => call($51, "test", [44]));

// table_copy.wast:2843
assert_trap(() => call($51, "test", [45]));

// table_copy.wast:2844
assert_trap(() => call($51, "test", [46]));

// table_copy.wast:2845
assert_trap(() => call($51, "test", [47]));

// table_copy.wast:2846
assert_trap(() => call($51, "test", [48]));

// table_copy.wast:2847
assert_trap(() => call($51, "test", [49]));

// table_copy.wast:2848
assert_trap(() => call($51, "test", [50]));

// table_copy.wast:2849
assert_trap(() => call($51, "test", [51]));

// table_copy.wast:2850
assert_trap(() => call($51, "test", [52]));

// table_copy.wast:2851
assert_trap(() => call($51, "test", [53]));

// table_copy.wast:2852
assert_trap(() => call($51, "test", [54]));

// table_copy.wast:2853
assert_trap(() => call($51, "test", [55]));

// table_copy.wast:2854
assert_trap(() => call($51, "test", [56]));

// table_copy.wast:2855
assert_trap(() => call($51, "test", [57]));

// table_copy.wast:2856
assert_trap(() => call($51, "test", [58]));

// table_copy.wast:2857
assert_trap(() => call($51, "test", [59]));

// table_copy.wast:2858
assert_trap(() => call($51, "test", [60]));

// table_copy.wast:2859
assert_trap(() => call($51, "test", [61]));

// table_copy.wast:2860
assert_trap(() => call($51, "test", [62]));

// table_copy.wast:2861
assert_trap(() => call($51, "test", [63]));

// table_copy.wast:2862
assert_trap(() => call($51, "test", [64]));

// table_copy.wast:2863
assert_trap(() => call($51, "test", [65]));

// table_copy.wast:2864
assert_trap(() => call($51, "test", [66]));

// table_copy.wast:2865
assert_trap(() => call($51, "test", [67]));

// table_copy.wast:2866
assert_trap(() => call($51, "test", [68]));

// table_copy.wast:2867
assert_trap(() => call($51, "test", [69]));

// table_copy.wast:2868
assert_trap(() => call($51, "test", [70]));

// table_copy.wast:2869
assert_trap(() => call($51, "test", [71]));

// table_copy.wast:2870
assert_trap(() => call($51, "test", [72]));

// table_copy.wast:2871
assert_trap(() => call($51, "test", [73]));

// table_copy.wast:2872
assert_trap(() => call($51, "test", [74]));

// table_copy.wast:2873
assert_trap(() => call($51, "test", [75]));

// table_copy.wast:2874
assert_trap(() => call($51, "test", [76]));

// table_copy.wast:2875
assert_trap(() => call($51, "test", [77]));

// table_copy.wast:2876
assert_trap(() => call($51, "test", [78]));

// table_copy.wast:2877
assert_trap(() => call($51, "test", [79]));

// table_copy.wast:2878
assert_trap(() => call($51, "test", [80]));

// table_copy.wast:2879
assert_trap(() => call($51, "test", [81]));

// table_copy.wast:2880
assert_trap(() => call($51, "test", [82]));

// table_copy.wast:2881
assert_trap(() => call($51, "test", [83]));

// table_copy.wast:2882
assert_trap(() => call($51, "test", [84]));

// table_copy.wast:2883
assert_trap(() => call($51, "test", [85]));

// table_copy.wast:2884
assert_trap(() => call($51, "test", [86]));

// table_copy.wast:2885
assert_trap(() => call($51, "test", [87]));

// table_copy.wast:2886
assert_trap(() => call($51, "test", [88]));

// table_copy.wast:2887
assert_trap(() => call($51, "test", [89]));

// table_copy.wast:2888
assert_trap(() => call($51, "test", [90]));

// table_copy.wast:2889
assert_trap(() => call($51, "test", [91]));

// table_copy.wast:2890
assert_trap(() => call($51, "test", [92]));

// table_copy.wast:2891
assert_trap(() => call($51, "test", [93]));

// table_copy.wast:2892
assert_trap(() => call($51, "test", [94]));

// table_copy.wast:2893
assert_trap(() => call($51, "test", [95]));

// table_copy.wast:2894
assert_trap(() => call($51, "test", [96]));

// table_copy.wast:2895
assert_trap(() => call($51, "test", [97]));

// table_copy.wast:2896
assert_trap(() => call($51, "test", [98]));

// table_copy.wast:2897
assert_trap(() => call($51, "test", [99]));

// table_copy.wast:2898
assert_trap(() => call($51, "test", [100]));

// table_copy.wast:2899
assert_trap(() => call($51, "test", [101]));

// table_copy.wast:2900
assert_trap(() => call($51, "test", [102]));

// table_copy.wast:2901
assert_trap(() => call($51, "test", [103]));

// table_copy.wast:2902
assert_trap(() => call($51, "test", [104]));

// table_copy.wast:2903
assert_trap(() => call($51, "test", [105]));

// table_copy.wast:2904
assert_trap(() => call($51, "test", [106]));

// table_copy.wast:2905
assert_trap(() => call($51, "test", [107]));

// table_copy.wast:2906
assert_trap(() => call($51, "test", [108]));

// table_copy.wast:2907
assert_trap(() => call($51, "test", [109]));

// table_copy.wast:2908
assert_trap(() => call($51, "test", [110]));

// table_copy.wast:2909
assert_trap(() => call($51, "test", [111]));

// table_copy.wast:2910
assert_return(() => call($51, "test", [112]), 0);

// table_copy.wast:2911
assert_return(() => call($51, "test", [113]), 1);

// table_copy.wast:2912
assert_return(() => call($51, "test", [114]), 2);

// table_copy.wast:2913
assert_return(() => call($51, "test", [115]), 3);

// table_copy.wast:2914
assert_return(() => call($51, "test", [116]), 4);

// table_copy.wast:2915
assert_return(() => call($51, "test", [117]), 5);

// table_copy.wast:2916
assert_return(() => call($51, "test", [118]), 6);

// table_copy.wast:2917
assert_return(() => call($51, "test", [119]), 7);

// table_copy.wast:2918
assert_return(() => call($51, "test", [120]), 8);

// table_copy.wast:2919
assert_return(() => call($51, "test", [121]), 9);

// table_copy.wast:2920
assert_return(() => call($51, "test", [122]), 10);

// table_copy.wast:2921
assert_return(() => call($51, "test", [123]), 11);

// table_copy.wast:2922
assert_return(() => call($51, "test", [124]), 12);

// table_copy.wast:2923
assert_return(() => call($51, "test", [125]), 13);

// table_copy.wast:2924
assert_return(() => call($51, "test", [126]), 14);

// table_copy.wast:2925
assert_return(() => call($51, "test", [127]), 15);

// table_copy.wast:2927
let $52 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x90\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x93\x80\x80\x80\x00\x12\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x02\x04\x87\x80\x80\x80\x00\x01\x70\x01\x80\x01\x80\x01\x07\xe4\x80\x80\x80\x00\x12\x02\x66\x30\x00\x00\x02\x66\x31\x00\x01\x02\x66\x32\x00\x02\x02\x66\x33\x00\x03\x02\x66\x34\x00\x04\x02\x66\x35\x00\x05\x02\x66\x36\x00\x06\x02\x66\x37\x00\x07\x02\x66\x38\x00\x08\x02\x66\x39\x00\x09\x03\x66\x31\x30\x00\x0a\x03\x66\x31\x31\x00\x0b\x03\x66\x31\x32\x00\x0c\x03\x66\x31\x33\x00\x0d\x03\x66\x31\x34\x00\x0e\x03\x66\x31\x35\x00\x0f\x04\x74\x65\x73\x74\x00\x10\x03\x72\x75\x6e\x00\x11\x09\x96\x80\x80\x80\x00\x01\x00\x41\x00\x0b\x10\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x0a\xae\x81\x80\x80\x00\x12\x84\x80\x80\x80\x00\x00\x41\x00\x0b\x84\x80\x80\x80\x00\x00\x41\x01\x0b\x84\x80\x80\x80\x00\x00\x41\x02\x0b\x84\x80\x80\x80\x00\x00\x41\x03\x0b\x84\x80\x80\x80\x00\x00\x41\x04\x0b\x84\x80\x80\x80\x00\x00\x41\x05\x0b\x84\x80\x80\x80\x00\x00\x41\x06\x0b\x84\x80\x80\x80\x00\x00\x41\x07\x0b\x84\x80\x80\x80\x00\x00\x41\x08\x0b\x84\x80\x80\x80\x00\x00\x41\x09\x0b\x84\x80\x80\x80\x00\x00\x41\x0a\x0b\x84\x80\x80\x80\x00\x00\x41\x0b\x0b\x84\x80\x80\x80\x00\x00\x41\x0c\x0b\x84\x80\x80\x80\x00\x00\x41\x0d\x0b\x84\x80\x80\x80\x00\x00\x41\x0e\x0b\x84\x80\x80\x80\x00\x00\x41\x0f\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x11\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b");

// table_copy.wast:2953
assert_trap(() => call($52, "run", [112, 0, -32]));

// table_copy.wast:2955
assert_return(() => call($52, "test", [0]), 0);

// table_copy.wast:2956
assert_return(() => call($52, "test", [1]), 1);

// table_copy.wast:2957
assert_return(() => call($52, "test", [2]), 2);

// table_copy.wast:2958
assert_return(() => call($52, "test", [3]), 3);

// table_copy.wast:2959
assert_return(() => call($52, "test", [4]), 4);

// table_copy.wast:2960
assert_return(() => call($52, "test", [5]), 5);

// table_copy.wast:2961
assert_return(() => call($52, "test", [6]), 6);

// table_copy.wast:2962
assert_return(() => call($52, "test", [7]), 7);

// table_copy.wast:2963
assert_return(() => call($52, "test", [8]), 8);

// table_copy.wast:2964
assert_return(() => call($52, "test", [9]), 9);

// table_copy.wast:2965
assert_return(() => call($52, "test", [10]), 10);

// table_copy.wast:2966
assert_return(() => call($52, "test", [11]), 11);

// table_copy.wast:2967
assert_return(() => call($52, "test", [12]), 12);

// table_copy.wast:2968
assert_return(() => call($52, "test", [13]), 13);

// table_copy.wast:2969
assert_return(() => call($52, "test", [14]), 14);

// table_copy.wast:2970
assert_return(() => call($52, "test", [15]), 15);

// table_copy.wast:2971
assert_trap(() => call($52, "test", [16]));

// table_copy.wast:2972
assert_trap(() => call($52, "test", [17]));

// table_copy.wast:2973
assert_trap(() => call($52, "test", [18]));

// table_copy.wast:2974
assert_trap(() => call($52, "test", [19]));

// table_copy.wast:2975
assert_trap(() => call($52, "test", [20]));

// table_copy.wast:2976
assert_trap(() => call($52, "test", [21]));

// table_copy.wast:2977
assert_trap(() => call($52, "test", [22]));

// table_copy.wast:2978
assert_trap(() => call($52, "test", [23]));

// table_copy.wast:2979
assert_trap(() => call($52, "test", [24]));

// table_copy.wast:2980
assert_trap(() => call($52, "test", [25]));

// table_copy.wast:2981
assert_trap(() => call($52, "test", [26]));

// table_copy.wast:2982
assert_trap(() => call($52, "test", [27]));

// table_copy.wast:2983
assert_trap(() => call($52, "test", [28]));

// table_copy.wast:2984
assert_trap(() => call($52, "test", [29]));

// table_copy.wast:2985
assert_trap(() => call($52, "test", [30]));

// table_copy.wast:2986
assert_trap(() => call($52, "test", [31]));

// table_copy.wast:2987
assert_trap(() => call($52, "test", [32]));

// table_copy.wast:2988
assert_trap(() => call($52, "test", [33]));

// table_copy.wast:2989
assert_trap(() => call($52, "test", [34]));

// table_copy.wast:2990
assert_trap(() => call($52, "test", [35]));

// table_copy.wast:2991
assert_trap(() => call($52, "test", [36]));

// table_copy.wast:2992
assert_trap(() => call($52, "test", [37]));

// table_copy.wast:2993
assert_trap(() => call($52, "test", [38]));

// table_copy.wast:2994
assert_trap(() => call($52, "test", [39]));

// table_copy.wast:2995
assert_trap(() => call($52, "test", [40]));

// table_copy.wast:2996
assert_trap(() => call($52, "test", [41]));

// table_copy.wast:2997
assert_trap(() => call($52, "test", [42]));

// table_copy.wast:2998
assert_trap(() => call($52, "test", [43]));

// table_copy.wast:2999
assert_trap(() => call($52, "test", [44]));

// table_copy.wast:3000
assert_trap(() => call($52, "test", [45]));

// table_copy.wast:3001
assert_trap(() => call($52, "test", [46]));

// table_copy.wast:3002
assert_trap(() => call($52, "test", [47]));

// table_copy.wast:3003
assert_trap(() => call($52, "test", [48]));

// table_copy.wast:3004
assert_trap(() => call($52, "test", [49]));

// table_copy.wast:3005
assert_trap(() => call($52, "test", [50]));

// table_copy.wast:3006
assert_trap(() => call($52, "test", [51]));

// table_copy.wast:3007
assert_trap(() => call($52, "test", [52]));

// table_copy.wast:3008
assert_trap(() => call($52, "test", [53]));

// table_copy.wast:3009
assert_trap(() => call($52, "test", [54]));

// table_copy.wast:3010
assert_trap(() => call($52, "test", [55]));

// table_copy.wast:3011
assert_trap(() => call($52, "test", [56]));

// table_copy.wast:3012
assert_trap(() => call($52, "test", [57]));

// table_copy.wast:3013
assert_trap(() => call($52, "test", [58]));

// table_copy.wast:3014
assert_trap(() => call($52, "test", [59]));

// table_copy.wast:3015
assert_trap(() => call($52, "test", [60]));

// table_copy.wast:3016
assert_trap(() => call($52, "test", [61]));

// table_copy.wast:3017
assert_trap(() => call($52, "test", [62]));

// table_copy.wast:3018
assert_trap(() => call($52, "test", [63]));

// table_copy.wast:3019
assert_trap(() => call($52, "test", [64]));

// table_copy.wast:3020
assert_trap(() => call($52, "test", [65]));

// table_copy.wast:3021
assert_trap(() => call($52, "test", [66]));

// table_copy.wast:3022
assert_trap(() => call($52, "test", [67]));

// table_copy.wast:3023
assert_trap(() => call($52, "test", [68]));

// table_copy.wast:3024
assert_trap(() => call($52, "test", [69]));

// table_copy.wast:3025
assert_trap(() => call($52, "test", [70]));

// table_copy.wast:3026
assert_trap(() => call($52, "test", [71]));

// table_copy.wast:3027
assert_trap(() => call($52, "test", [72]));

// table_copy.wast:3028
assert_trap(() => call($52, "test", [73]));

// table_copy.wast:3029
assert_trap(() => call($52, "test", [74]));

// table_copy.wast:3030
assert_trap(() => call($52, "test", [75]));

// table_copy.wast:3031
assert_trap(() => call($52, "test", [76]));

// table_copy.wast:3032
assert_trap(() => call($52, "test", [77]));

// table_copy.wast:3033
assert_trap(() => call($52, "test", [78]));

// table_copy.wast:3034
assert_trap(() => call($52, "test", [79]));

// table_copy.wast:3035
assert_trap(() => call($52, "test", [80]));

// table_copy.wast:3036
assert_trap(() => call($52, "test", [81]));

// table_copy.wast:3037
assert_trap(() => call($52, "test", [82]));

// table_copy.wast:3038
assert_trap(() => call($52, "test", [83]));

// table_copy.wast:3039
assert_trap(() => call($52, "test", [84]));

// table_copy.wast:3040
assert_trap(() => call($52, "test", [85]));

// table_copy.wast:3041
assert_trap(() => call($52, "test", [86]));

// table_copy.wast:3042
assert_trap(() => call($52, "test", [87]));

// table_copy.wast:3043
assert_trap(() => call($52, "test", [88]));

// table_copy.wast:3044
assert_trap(() => call($52, "test", [89]));

// table_copy.wast:3045
assert_trap(() => call($52, "test", [90]));

// table_copy.wast:3046
assert_trap(() => call($52, "test", [91]));

// table_copy.wast:3047
assert_trap(() => call($52, "test", [92]));

// table_copy.wast:3048
assert_trap(() => call($52, "test", [93]));

// table_copy.wast:3049
assert_trap(() => call($52, "test", [94]));

// table_copy.wast:3050
assert_trap(() => call($52, "test", [95]));

// table_copy.wast:3051
assert_trap(() => call($52, "test", [96]));

// table_copy.wast:3052
assert_trap(() => call($52, "test", [97]));

// table_copy.wast:3053
assert_trap(() => call($52, "test", [98]));

// table_copy.wast:3054
assert_trap(() => call($52, "test", [99]));

// table_copy.wast:3055
assert_trap(() => call($52, "test", [100]));

// table_copy.wast:3056
assert_trap(() => call($52, "test", [101]));

// table_copy.wast:3057
assert_trap(() => call($52, "test", [102]));

// table_copy.wast:3058
assert_trap(() => call($52, "test", [103]));

// table_copy.wast:3059
assert_trap(() => call($52, "test", [104]));

// table_copy.wast:3060
assert_trap(() => call($52, "test", [105]));

// table_copy.wast:3061
assert_trap(() => call($52, "test", [106]));

// table_copy.wast:3062
assert_trap(() => call($52, "test", [107]));

// table_copy.wast:3063
assert_trap(() => call($52, "test", [108]));

// table_copy.wast:3064
assert_trap(() => call($52, "test", [109]));

// table_copy.wast:3065
assert_trap(() => call($52, "test", [110]));

// table_copy.wast:3066
assert_trap(() => call($52, "test", [111]));

// table_copy.wast:3067
assert_trap(() => call($52, "test", [112]));

// table_copy.wast:3068
assert_trap(() => call($52, "test", [113]));

// table_copy.wast:3069
assert_trap(() => call($52, "test", [114]));

// table_copy.wast:3070
assert_trap(() => call($52, "test", [115]));

// table_copy.wast:3071
assert_trap(() => call($52, "test", [116]));

// table_copy.wast:3072
assert_trap(() => call($52, "test", [117]));

// table_copy.wast:3073
assert_trap(() => call($52, "test", [118]));

// table_copy.wast:3074
assert_trap(() => call($52, "test", [119]));

// table_copy.wast:3075
assert_trap(() => call($52, "test", [120]));

// table_copy.wast:3076
assert_trap(() => call($52, "test", [121]));

// table_copy.wast:3077
assert_trap(() => call($52, "test", [122]));

// table_copy.wast:3078
assert_trap(() => call($52, "test", [123]));

// table_copy.wast:3079
assert_trap(() => call($52, "test", [124]));

// table_copy.wast:3080
assert_trap(() => call($52, "test", [125]));

// table_copy.wast:3081
assert_trap(() => call($52, "test", [126]));

// table_copy.wast:3082
assert_trap(() => call($52, "test", [127]));
reinitializeRegistry();
})();
