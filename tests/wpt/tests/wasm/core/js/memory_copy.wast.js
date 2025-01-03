(function memory_copy_wast_js() {

// memory_copy.wast:6
let $1 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x9c\x80\x80\x80\x00\x03\x07\x6d\x65\x6d\x6f\x72\x79\x30\x02\x00\x04\x74\x65\x73\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x95\x80\x80\x80\x00\x02\x83\x80\x80\x80\x00\x00\x01\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x94\x80\x80\x80\x00\x02\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06");

// memory_copy.wast:15
run(() => call($1, "test", []));

// memory_copy.wast:17
assert_return(() => call($1, "load8_u", [0]), 0);

// memory_copy.wast:18
assert_return(() => call($1, "load8_u", [1]), 0);

// memory_copy.wast:19
assert_return(() => call($1, "load8_u", [2]), 3);

// memory_copy.wast:20
assert_return(() => call($1, "load8_u", [3]), 1);

// memory_copy.wast:21
assert_return(() => call($1, "load8_u", [4]), 4);

// memory_copy.wast:22
assert_return(() => call($1, "load8_u", [5]), 1);

// memory_copy.wast:23
assert_return(() => call($1, "load8_u", [6]), 0);

// memory_copy.wast:24
assert_return(() => call($1, "load8_u", [7]), 0);

// memory_copy.wast:25
assert_return(() => call($1, "load8_u", [8]), 0);

// memory_copy.wast:26
assert_return(() => call($1, "load8_u", [9]), 0);

// memory_copy.wast:27
assert_return(() => call($1, "load8_u", [10]), 0);

// memory_copy.wast:28
assert_return(() => call($1, "load8_u", [11]), 0);

// memory_copy.wast:29
assert_return(() => call($1, "load8_u", [12]), 7);

// memory_copy.wast:30
assert_return(() => call($1, "load8_u", [13]), 5);

// memory_copy.wast:31
assert_return(() => call($1, "load8_u", [14]), 2);

// memory_copy.wast:32
assert_return(() => call($1, "load8_u", [15]), 3);

// memory_copy.wast:33
assert_return(() => call($1, "load8_u", [16]), 6);

// memory_copy.wast:34
assert_return(() => call($1, "load8_u", [17]), 0);

// memory_copy.wast:35
assert_return(() => call($1, "load8_u", [18]), 0);

// memory_copy.wast:36
assert_return(() => call($1, "load8_u", [19]), 0);

// memory_copy.wast:37
assert_return(() => call($1, "load8_u", [20]), 0);

// memory_copy.wast:38
assert_return(() => call($1, "load8_u", [21]), 0);

// memory_copy.wast:39
assert_return(() => call($1, "load8_u", [22]), 0);

// memory_copy.wast:40
assert_return(() => call($1, "load8_u", [23]), 0);

// memory_copy.wast:41
assert_return(() => call($1, "load8_u", [24]), 0);

// memory_copy.wast:42
assert_return(() => call($1, "load8_u", [25]), 0);

// memory_copy.wast:43
assert_return(() => call($1, "load8_u", [26]), 0);

// memory_copy.wast:44
assert_return(() => call($1, "load8_u", [27]), 0);

// memory_copy.wast:45
assert_return(() => call($1, "load8_u", [28]), 0);

// memory_copy.wast:46
assert_return(() => call($1, "load8_u", [29]), 0);

// memory_copy.wast:48
let $2 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x9c\x80\x80\x80\x00\x03\x07\x6d\x65\x6d\x6f\x72\x79\x30\x02\x00\x04\x74\x65\x73\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x41\x0d\x41\x02\x41\x03\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x94\x80\x80\x80\x00\x02\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06");

// memory_copy.wast:57
run(() => call($2, "test", []));

// memory_copy.wast:59
assert_return(() => call($2, "load8_u", [0]), 0);

// memory_copy.wast:60
assert_return(() => call($2, "load8_u", [1]), 0);

// memory_copy.wast:61
assert_return(() => call($2, "load8_u", [2]), 3);

// memory_copy.wast:62
assert_return(() => call($2, "load8_u", [3]), 1);

// memory_copy.wast:63
assert_return(() => call($2, "load8_u", [4]), 4);

// memory_copy.wast:64
assert_return(() => call($2, "load8_u", [5]), 1);

// memory_copy.wast:65
assert_return(() => call($2, "load8_u", [6]), 0);

// memory_copy.wast:66
assert_return(() => call($2, "load8_u", [7]), 0);

// memory_copy.wast:67
assert_return(() => call($2, "load8_u", [8]), 0);

// memory_copy.wast:68
assert_return(() => call($2, "load8_u", [9]), 0);

// memory_copy.wast:69
assert_return(() => call($2, "load8_u", [10]), 0);

// memory_copy.wast:70
assert_return(() => call($2, "load8_u", [11]), 0);

// memory_copy.wast:71
assert_return(() => call($2, "load8_u", [12]), 7);

// memory_copy.wast:72
assert_return(() => call($2, "load8_u", [13]), 3);

// memory_copy.wast:73
assert_return(() => call($2, "load8_u", [14]), 1);

// memory_copy.wast:74
assert_return(() => call($2, "load8_u", [15]), 4);

// memory_copy.wast:75
assert_return(() => call($2, "load8_u", [16]), 6);

// memory_copy.wast:76
assert_return(() => call($2, "load8_u", [17]), 0);

// memory_copy.wast:77
assert_return(() => call($2, "load8_u", [18]), 0);

// memory_copy.wast:78
assert_return(() => call($2, "load8_u", [19]), 0);

// memory_copy.wast:79
assert_return(() => call($2, "load8_u", [20]), 0);

// memory_copy.wast:80
assert_return(() => call($2, "load8_u", [21]), 0);

// memory_copy.wast:81
assert_return(() => call($2, "load8_u", [22]), 0);

// memory_copy.wast:82
assert_return(() => call($2, "load8_u", [23]), 0);

// memory_copy.wast:83
assert_return(() => call($2, "load8_u", [24]), 0);

// memory_copy.wast:84
assert_return(() => call($2, "load8_u", [25]), 0);

// memory_copy.wast:85
assert_return(() => call($2, "load8_u", [26]), 0);

// memory_copy.wast:86
assert_return(() => call($2, "load8_u", [27]), 0);

// memory_copy.wast:87
assert_return(() => call($2, "load8_u", [28]), 0);

// memory_copy.wast:88
assert_return(() => call($2, "load8_u", [29]), 0);

// memory_copy.wast:90
let $3 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x9c\x80\x80\x80\x00\x03\x07\x6d\x65\x6d\x6f\x72\x79\x30\x02\x00\x04\x74\x65\x73\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x41\x19\x41\x0f\x41\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x94\x80\x80\x80\x00\x02\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06");

// memory_copy.wast:99
run(() => call($3, "test", []));

// memory_copy.wast:101
assert_return(() => call($3, "load8_u", [0]), 0);

// memory_copy.wast:102
assert_return(() => call($3, "load8_u", [1]), 0);

// memory_copy.wast:103
assert_return(() => call($3, "load8_u", [2]), 3);

// memory_copy.wast:104
assert_return(() => call($3, "load8_u", [3]), 1);

// memory_copy.wast:105
assert_return(() => call($3, "load8_u", [4]), 4);

// memory_copy.wast:106
assert_return(() => call($3, "load8_u", [5]), 1);

// memory_copy.wast:107
assert_return(() => call($3, "load8_u", [6]), 0);

// memory_copy.wast:108
assert_return(() => call($3, "load8_u", [7]), 0);

// memory_copy.wast:109
assert_return(() => call($3, "load8_u", [8]), 0);

// memory_copy.wast:110
assert_return(() => call($3, "load8_u", [9]), 0);

// memory_copy.wast:111
assert_return(() => call($3, "load8_u", [10]), 0);

// memory_copy.wast:112
assert_return(() => call($3, "load8_u", [11]), 0);

// memory_copy.wast:113
assert_return(() => call($3, "load8_u", [12]), 7);

// memory_copy.wast:114
assert_return(() => call($3, "load8_u", [13]), 5);

// memory_copy.wast:115
assert_return(() => call($3, "load8_u", [14]), 2);

// memory_copy.wast:116
assert_return(() => call($3, "load8_u", [15]), 3);

// memory_copy.wast:117
assert_return(() => call($3, "load8_u", [16]), 6);

// memory_copy.wast:118
assert_return(() => call($3, "load8_u", [17]), 0);

// memory_copy.wast:119
assert_return(() => call($3, "load8_u", [18]), 0);

// memory_copy.wast:120
assert_return(() => call($3, "load8_u", [19]), 0);

// memory_copy.wast:121
assert_return(() => call($3, "load8_u", [20]), 0);

// memory_copy.wast:122
assert_return(() => call($3, "load8_u", [21]), 0);

// memory_copy.wast:123
assert_return(() => call($3, "load8_u", [22]), 0);

// memory_copy.wast:124
assert_return(() => call($3, "load8_u", [23]), 0);

// memory_copy.wast:125
assert_return(() => call($3, "load8_u", [24]), 0);

// memory_copy.wast:126
assert_return(() => call($3, "load8_u", [25]), 3);

// memory_copy.wast:127
assert_return(() => call($3, "load8_u", [26]), 6);

// memory_copy.wast:128
assert_return(() => call($3, "load8_u", [27]), 0);

// memory_copy.wast:129
assert_return(() => call($3, "load8_u", [28]), 0);

// memory_copy.wast:130
assert_return(() => call($3, "load8_u", [29]), 0);

// memory_copy.wast:132
let $4 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x9c\x80\x80\x80\x00\x03\x07\x6d\x65\x6d\x6f\x72\x79\x30\x02\x00\x04\x74\x65\x73\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x41\x0d\x41\x19\x41\x03\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x94\x80\x80\x80\x00\x02\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06");

// memory_copy.wast:141
run(() => call($4, "test", []));

// memory_copy.wast:143
assert_return(() => call($4, "load8_u", [0]), 0);

// memory_copy.wast:144
assert_return(() => call($4, "load8_u", [1]), 0);

// memory_copy.wast:145
assert_return(() => call($4, "load8_u", [2]), 3);

// memory_copy.wast:146
assert_return(() => call($4, "load8_u", [3]), 1);

// memory_copy.wast:147
assert_return(() => call($4, "load8_u", [4]), 4);

// memory_copy.wast:148
assert_return(() => call($4, "load8_u", [5]), 1);

// memory_copy.wast:149
assert_return(() => call($4, "load8_u", [6]), 0);

// memory_copy.wast:150
assert_return(() => call($4, "load8_u", [7]), 0);

// memory_copy.wast:151
assert_return(() => call($4, "load8_u", [8]), 0);

// memory_copy.wast:152
assert_return(() => call($4, "load8_u", [9]), 0);

// memory_copy.wast:153
assert_return(() => call($4, "load8_u", [10]), 0);

// memory_copy.wast:154
assert_return(() => call($4, "load8_u", [11]), 0);

// memory_copy.wast:155
assert_return(() => call($4, "load8_u", [12]), 7);

// memory_copy.wast:156
assert_return(() => call($4, "load8_u", [13]), 0);

// memory_copy.wast:157
assert_return(() => call($4, "load8_u", [14]), 0);

// memory_copy.wast:158
assert_return(() => call($4, "load8_u", [15]), 0);

// memory_copy.wast:159
assert_return(() => call($4, "load8_u", [16]), 6);

// memory_copy.wast:160
assert_return(() => call($4, "load8_u", [17]), 0);

// memory_copy.wast:161
assert_return(() => call($4, "load8_u", [18]), 0);

// memory_copy.wast:162
assert_return(() => call($4, "load8_u", [19]), 0);

// memory_copy.wast:163
assert_return(() => call($4, "load8_u", [20]), 0);

// memory_copy.wast:164
assert_return(() => call($4, "load8_u", [21]), 0);

// memory_copy.wast:165
assert_return(() => call($4, "load8_u", [22]), 0);

// memory_copy.wast:166
assert_return(() => call($4, "load8_u", [23]), 0);

// memory_copy.wast:167
assert_return(() => call($4, "load8_u", [24]), 0);

// memory_copy.wast:168
assert_return(() => call($4, "load8_u", [25]), 0);

// memory_copy.wast:169
assert_return(() => call($4, "load8_u", [26]), 0);

// memory_copy.wast:170
assert_return(() => call($4, "load8_u", [27]), 0);

// memory_copy.wast:171
assert_return(() => call($4, "load8_u", [28]), 0);

// memory_copy.wast:172
assert_return(() => call($4, "load8_u", [29]), 0);

// memory_copy.wast:174
let $5 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x9c\x80\x80\x80\x00\x03\x07\x6d\x65\x6d\x6f\x72\x79\x30\x02\x00\x04\x74\x65\x73\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x41\x14\x41\x16\x41\x04\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x94\x80\x80\x80\x00\x02\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06");

// memory_copy.wast:183
run(() => call($5, "test", []));

// memory_copy.wast:185
assert_return(() => call($5, "load8_u", [0]), 0);

// memory_copy.wast:186
assert_return(() => call($5, "load8_u", [1]), 0);

// memory_copy.wast:187
assert_return(() => call($5, "load8_u", [2]), 3);

// memory_copy.wast:188
assert_return(() => call($5, "load8_u", [3]), 1);

// memory_copy.wast:189
assert_return(() => call($5, "load8_u", [4]), 4);

// memory_copy.wast:190
assert_return(() => call($5, "load8_u", [5]), 1);

// memory_copy.wast:191
assert_return(() => call($5, "load8_u", [6]), 0);

// memory_copy.wast:192
assert_return(() => call($5, "load8_u", [7]), 0);

// memory_copy.wast:193
assert_return(() => call($5, "load8_u", [8]), 0);

// memory_copy.wast:194
assert_return(() => call($5, "load8_u", [9]), 0);

// memory_copy.wast:195
assert_return(() => call($5, "load8_u", [10]), 0);

// memory_copy.wast:196
assert_return(() => call($5, "load8_u", [11]), 0);

// memory_copy.wast:197
assert_return(() => call($5, "load8_u", [12]), 7);

// memory_copy.wast:198
assert_return(() => call($5, "load8_u", [13]), 5);

// memory_copy.wast:199
assert_return(() => call($5, "load8_u", [14]), 2);

// memory_copy.wast:200
assert_return(() => call($5, "load8_u", [15]), 3);

// memory_copy.wast:201
assert_return(() => call($5, "load8_u", [16]), 6);

// memory_copy.wast:202
assert_return(() => call($5, "load8_u", [17]), 0);

// memory_copy.wast:203
assert_return(() => call($5, "load8_u", [18]), 0);

// memory_copy.wast:204
assert_return(() => call($5, "load8_u", [19]), 0);

// memory_copy.wast:205
assert_return(() => call($5, "load8_u", [20]), 0);

// memory_copy.wast:206
assert_return(() => call($5, "load8_u", [21]), 0);

// memory_copy.wast:207
assert_return(() => call($5, "load8_u", [22]), 0);

// memory_copy.wast:208
assert_return(() => call($5, "load8_u", [23]), 0);

// memory_copy.wast:209
assert_return(() => call($5, "load8_u", [24]), 0);

// memory_copy.wast:210
assert_return(() => call($5, "load8_u", [25]), 0);

// memory_copy.wast:211
assert_return(() => call($5, "load8_u", [26]), 0);

// memory_copy.wast:212
assert_return(() => call($5, "load8_u", [27]), 0);

// memory_copy.wast:213
assert_return(() => call($5, "load8_u", [28]), 0);

// memory_copy.wast:214
assert_return(() => call($5, "load8_u", [29]), 0);

// memory_copy.wast:216
let $6 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x9c\x80\x80\x80\x00\x03\x07\x6d\x65\x6d\x6f\x72\x79\x30\x02\x00\x04\x74\x65\x73\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x41\x19\x41\x01\x41\x03\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x94\x80\x80\x80\x00\x02\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06");

// memory_copy.wast:225
run(() => call($6, "test", []));

// memory_copy.wast:227
assert_return(() => call($6, "load8_u", [0]), 0);

// memory_copy.wast:228
assert_return(() => call($6, "load8_u", [1]), 0);

// memory_copy.wast:229
assert_return(() => call($6, "load8_u", [2]), 3);

// memory_copy.wast:230
assert_return(() => call($6, "load8_u", [3]), 1);

// memory_copy.wast:231
assert_return(() => call($6, "load8_u", [4]), 4);

// memory_copy.wast:232
assert_return(() => call($6, "load8_u", [5]), 1);

// memory_copy.wast:233
assert_return(() => call($6, "load8_u", [6]), 0);

// memory_copy.wast:234
assert_return(() => call($6, "load8_u", [7]), 0);

// memory_copy.wast:235
assert_return(() => call($6, "load8_u", [8]), 0);

// memory_copy.wast:236
assert_return(() => call($6, "load8_u", [9]), 0);

// memory_copy.wast:237
assert_return(() => call($6, "load8_u", [10]), 0);

// memory_copy.wast:238
assert_return(() => call($6, "load8_u", [11]), 0);

// memory_copy.wast:239
assert_return(() => call($6, "load8_u", [12]), 7);

// memory_copy.wast:240
assert_return(() => call($6, "load8_u", [13]), 5);

// memory_copy.wast:241
assert_return(() => call($6, "load8_u", [14]), 2);

// memory_copy.wast:242
assert_return(() => call($6, "load8_u", [15]), 3);

// memory_copy.wast:243
assert_return(() => call($6, "load8_u", [16]), 6);

// memory_copy.wast:244
assert_return(() => call($6, "load8_u", [17]), 0);

// memory_copy.wast:245
assert_return(() => call($6, "load8_u", [18]), 0);

// memory_copy.wast:246
assert_return(() => call($6, "load8_u", [19]), 0);

// memory_copy.wast:247
assert_return(() => call($6, "load8_u", [20]), 0);

// memory_copy.wast:248
assert_return(() => call($6, "load8_u", [21]), 0);

// memory_copy.wast:249
assert_return(() => call($6, "load8_u", [22]), 0);

// memory_copy.wast:250
assert_return(() => call($6, "load8_u", [23]), 0);

// memory_copy.wast:251
assert_return(() => call($6, "load8_u", [24]), 0);

// memory_copy.wast:252
assert_return(() => call($6, "load8_u", [25]), 0);

// memory_copy.wast:253
assert_return(() => call($6, "load8_u", [26]), 3);

// memory_copy.wast:254
assert_return(() => call($6, "load8_u", [27]), 1);

// memory_copy.wast:255
assert_return(() => call($6, "load8_u", [28]), 0);

// memory_copy.wast:256
assert_return(() => call($6, "load8_u", [29]), 0);

// memory_copy.wast:258
let $7 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x9c\x80\x80\x80\x00\x03\x07\x6d\x65\x6d\x6f\x72\x79\x30\x02\x00\x04\x74\x65\x73\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x41\x0a\x41\x0c\x41\x07\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x94\x80\x80\x80\x00\x02\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06");

// memory_copy.wast:267
run(() => call($7, "test", []));

// memory_copy.wast:269
assert_return(() => call($7, "load8_u", [0]), 0);

// memory_copy.wast:270
assert_return(() => call($7, "load8_u", [1]), 0);

// memory_copy.wast:271
assert_return(() => call($7, "load8_u", [2]), 3);

// memory_copy.wast:272
assert_return(() => call($7, "load8_u", [3]), 1);

// memory_copy.wast:273
assert_return(() => call($7, "load8_u", [4]), 4);

// memory_copy.wast:274
assert_return(() => call($7, "load8_u", [5]), 1);

// memory_copy.wast:275
assert_return(() => call($7, "load8_u", [6]), 0);

// memory_copy.wast:276
assert_return(() => call($7, "load8_u", [7]), 0);

// memory_copy.wast:277
assert_return(() => call($7, "load8_u", [8]), 0);

// memory_copy.wast:278
assert_return(() => call($7, "load8_u", [9]), 0);

// memory_copy.wast:279
assert_return(() => call($7, "load8_u", [10]), 7);

// memory_copy.wast:280
assert_return(() => call($7, "load8_u", [11]), 5);

// memory_copy.wast:281
assert_return(() => call($7, "load8_u", [12]), 2);

// memory_copy.wast:282
assert_return(() => call($7, "load8_u", [13]), 3);

// memory_copy.wast:283
assert_return(() => call($7, "load8_u", [14]), 6);

// memory_copy.wast:284
assert_return(() => call($7, "load8_u", [15]), 0);

// memory_copy.wast:285
assert_return(() => call($7, "load8_u", [16]), 0);

// memory_copy.wast:286
assert_return(() => call($7, "load8_u", [17]), 0);

// memory_copy.wast:287
assert_return(() => call($7, "load8_u", [18]), 0);

// memory_copy.wast:288
assert_return(() => call($7, "load8_u", [19]), 0);

// memory_copy.wast:289
assert_return(() => call($7, "load8_u", [20]), 0);

// memory_copy.wast:290
assert_return(() => call($7, "load8_u", [21]), 0);

// memory_copy.wast:291
assert_return(() => call($7, "load8_u", [22]), 0);

// memory_copy.wast:292
assert_return(() => call($7, "load8_u", [23]), 0);

// memory_copy.wast:293
assert_return(() => call($7, "load8_u", [24]), 0);

// memory_copy.wast:294
assert_return(() => call($7, "load8_u", [25]), 0);

// memory_copy.wast:295
assert_return(() => call($7, "load8_u", [26]), 0);

// memory_copy.wast:296
assert_return(() => call($7, "load8_u", [27]), 0);

// memory_copy.wast:297
assert_return(() => call($7, "load8_u", [28]), 0);

// memory_copy.wast:298
assert_return(() => call($7, "load8_u", [29]), 0);

// memory_copy.wast:300
let $8 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x9c\x80\x80\x80\x00\x03\x07\x6d\x65\x6d\x6f\x72\x79\x30\x02\x00\x04\x74\x65\x73\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x41\x0c\x41\x0a\x41\x07\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x94\x80\x80\x80\x00\x02\x00\x41\x02\x0b\x04\x03\x01\x04\x01\x00\x41\x0c\x0b\x05\x07\x05\x02\x03\x06");

// memory_copy.wast:309
run(() => call($8, "test", []));

// memory_copy.wast:311
assert_return(() => call($8, "load8_u", [0]), 0);

// memory_copy.wast:312
assert_return(() => call($8, "load8_u", [1]), 0);

// memory_copy.wast:313
assert_return(() => call($8, "load8_u", [2]), 3);

// memory_copy.wast:314
assert_return(() => call($8, "load8_u", [3]), 1);

// memory_copy.wast:315
assert_return(() => call($8, "load8_u", [4]), 4);

// memory_copy.wast:316
assert_return(() => call($8, "load8_u", [5]), 1);

// memory_copy.wast:317
assert_return(() => call($8, "load8_u", [6]), 0);

// memory_copy.wast:318
assert_return(() => call($8, "load8_u", [7]), 0);

// memory_copy.wast:319
assert_return(() => call($8, "load8_u", [8]), 0);

// memory_copy.wast:320
assert_return(() => call($8, "load8_u", [9]), 0);

// memory_copy.wast:321
assert_return(() => call($8, "load8_u", [10]), 0);

// memory_copy.wast:322
assert_return(() => call($8, "load8_u", [11]), 0);

// memory_copy.wast:323
assert_return(() => call($8, "load8_u", [12]), 0);

// memory_copy.wast:324
assert_return(() => call($8, "load8_u", [13]), 0);

// memory_copy.wast:325
assert_return(() => call($8, "load8_u", [14]), 7);

// memory_copy.wast:326
assert_return(() => call($8, "load8_u", [15]), 5);

// memory_copy.wast:327
assert_return(() => call($8, "load8_u", [16]), 2);

// memory_copy.wast:328
assert_return(() => call($8, "load8_u", [17]), 3);

// memory_copy.wast:329
assert_return(() => call($8, "load8_u", [18]), 6);

// memory_copy.wast:330
assert_return(() => call($8, "load8_u", [19]), 0);

// memory_copy.wast:331
assert_return(() => call($8, "load8_u", [20]), 0);

// memory_copy.wast:332
assert_return(() => call($8, "load8_u", [21]), 0);

// memory_copy.wast:333
assert_return(() => call($8, "load8_u", [22]), 0);

// memory_copy.wast:334
assert_return(() => call($8, "load8_u", [23]), 0);

// memory_copy.wast:335
assert_return(() => call($8, "load8_u", [24]), 0);

// memory_copy.wast:336
assert_return(() => call($8, "load8_u", [25]), 0);

// memory_copy.wast:337
assert_return(() => call($8, "load8_u", [26]), 0);

// memory_copy.wast:338
assert_return(() => call($8, "load8_u", [27]), 0);

// memory_copy.wast:339
assert_return(() => call($8, "load8_u", [28]), 0);

// memory_copy.wast:340
assert_return(() => call($8, "load8_u", [29]), 0);

// memory_copy.wast:342
let $9 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9a\x80\x80\x80\x00\x01\x00\x41\x00\x0b\x14\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

// memory_copy.wast:350
assert_trap(() => call($9, "run", [65_516, 0, 40]));

// memory_copy.wast:353
assert_return(() => call($9, "load8_u", [0]), 0);

// memory_copy.wast:354
assert_return(() => call($9, "load8_u", [1]), 1);

// memory_copy.wast:355
assert_return(() => call($9, "load8_u", [2]), 2);

// memory_copy.wast:356
assert_return(() => call($9, "load8_u", [3]), 3);

// memory_copy.wast:357
assert_return(() => call($9, "load8_u", [4]), 4);

// memory_copy.wast:358
assert_return(() => call($9, "load8_u", [5]), 5);

// memory_copy.wast:359
assert_return(() => call($9, "load8_u", [6]), 6);

// memory_copy.wast:360
assert_return(() => call($9, "load8_u", [7]), 7);

// memory_copy.wast:361
assert_return(() => call($9, "load8_u", [8]), 8);

// memory_copy.wast:362
assert_return(() => call($9, "load8_u", [9]), 9);

// memory_copy.wast:363
assert_return(() => call($9, "load8_u", [10]), 10);

// memory_copy.wast:364
assert_return(() => call($9, "load8_u", [11]), 11);

// memory_copy.wast:365
assert_return(() => call($9, "load8_u", [12]), 12);

// memory_copy.wast:366
assert_return(() => call($9, "load8_u", [13]), 13);

// memory_copy.wast:367
assert_return(() => call($9, "load8_u", [14]), 14);

// memory_copy.wast:368
assert_return(() => call($9, "load8_u", [15]), 15);

// memory_copy.wast:369
assert_return(() => call($9, "load8_u", [16]), 16);

// memory_copy.wast:370
assert_return(() => call($9, "load8_u", [17]), 17);

// memory_copy.wast:371
assert_return(() => call($9, "load8_u", [18]), 18);

// memory_copy.wast:372
assert_return(() => call($9, "load8_u", [19]), 19);

// memory_copy.wast:373
assert_return(() => call($9, "load8_u", [218]), 0);

// memory_copy.wast:374
assert_return(() => call($9, "load8_u", [417]), 0);

// memory_copy.wast:375
assert_return(() => call($9, "load8_u", [616]), 0);

// memory_copy.wast:376
assert_return(() => call($9, "load8_u", [815]), 0);

// memory_copy.wast:377
assert_return(() => call($9, "load8_u", [1_014]), 0);

// memory_copy.wast:378
assert_return(() => call($9, "load8_u", [1_213]), 0);

// memory_copy.wast:379
assert_return(() => call($9, "load8_u", [1_412]), 0);

// memory_copy.wast:380
assert_return(() => call($9, "load8_u", [1_611]), 0);

// memory_copy.wast:381
assert_return(() => call($9, "load8_u", [1_810]), 0);

// memory_copy.wast:382
assert_return(() => call($9, "load8_u", [2_009]), 0);

// memory_copy.wast:383
assert_return(() => call($9, "load8_u", [2_208]), 0);

// memory_copy.wast:384
assert_return(() => call($9, "load8_u", [2_407]), 0);

// memory_copy.wast:385
assert_return(() => call($9, "load8_u", [2_606]), 0);

// memory_copy.wast:386
assert_return(() => call($9, "load8_u", [2_805]), 0);

// memory_copy.wast:387
assert_return(() => call($9, "load8_u", [3_004]), 0);

// memory_copy.wast:388
assert_return(() => call($9, "load8_u", [3_203]), 0);

// memory_copy.wast:389
assert_return(() => call($9, "load8_u", [3_402]), 0);

// memory_copy.wast:390
assert_return(() => call($9, "load8_u", [3_601]), 0);

// memory_copy.wast:391
assert_return(() => call($9, "load8_u", [3_800]), 0);

// memory_copy.wast:392
assert_return(() => call($9, "load8_u", [3_999]), 0);

// memory_copy.wast:393
assert_return(() => call($9, "load8_u", [4_198]), 0);

// memory_copy.wast:394
assert_return(() => call($9, "load8_u", [4_397]), 0);

// memory_copy.wast:395
assert_return(() => call($9, "load8_u", [4_596]), 0);

// memory_copy.wast:396
assert_return(() => call($9, "load8_u", [4_795]), 0);

// memory_copy.wast:397
assert_return(() => call($9, "load8_u", [4_994]), 0);

// memory_copy.wast:398
assert_return(() => call($9, "load8_u", [5_193]), 0);

// memory_copy.wast:399
assert_return(() => call($9, "load8_u", [5_392]), 0);

// memory_copy.wast:400
assert_return(() => call($9, "load8_u", [5_591]), 0);

// memory_copy.wast:401
assert_return(() => call($9, "load8_u", [5_790]), 0);

// memory_copy.wast:402
assert_return(() => call($9, "load8_u", [5_989]), 0);

// memory_copy.wast:403
assert_return(() => call($9, "load8_u", [6_188]), 0);

// memory_copy.wast:404
assert_return(() => call($9, "load8_u", [6_387]), 0);

// memory_copy.wast:405
assert_return(() => call($9, "load8_u", [6_586]), 0);

// memory_copy.wast:406
assert_return(() => call($9, "load8_u", [6_785]), 0);

// memory_copy.wast:407
assert_return(() => call($9, "load8_u", [6_984]), 0);

// memory_copy.wast:408
assert_return(() => call($9, "load8_u", [7_183]), 0);

// memory_copy.wast:409
assert_return(() => call($9, "load8_u", [7_382]), 0);

// memory_copy.wast:410
assert_return(() => call($9, "load8_u", [7_581]), 0);

// memory_copy.wast:411
assert_return(() => call($9, "load8_u", [7_780]), 0);

// memory_copy.wast:412
assert_return(() => call($9, "load8_u", [7_979]), 0);

// memory_copy.wast:413
assert_return(() => call($9, "load8_u", [8_178]), 0);

// memory_copy.wast:414
assert_return(() => call($9, "load8_u", [8_377]), 0);

// memory_copy.wast:415
assert_return(() => call($9, "load8_u", [8_576]), 0);

// memory_copy.wast:416
assert_return(() => call($9, "load8_u", [8_775]), 0);

// memory_copy.wast:417
assert_return(() => call($9, "load8_u", [8_974]), 0);

// memory_copy.wast:418
assert_return(() => call($9, "load8_u", [9_173]), 0);

// memory_copy.wast:419
assert_return(() => call($9, "load8_u", [9_372]), 0);

// memory_copy.wast:420
assert_return(() => call($9, "load8_u", [9_571]), 0);

// memory_copy.wast:421
assert_return(() => call($9, "load8_u", [9_770]), 0);

// memory_copy.wast:422
assert_return(() => call($9, "load8_u", [9_969]), 0);

// memory_copy.wast:423
assert_return(() => call($9, "load8_u", [10_168]), 0);

// memory_copy.wast:424
assert_return(() => call($9, "load8_u", [10_367]), 0);

// memory_copy.wast:425
assert_return(() => call($9, "load8_u", [10_566]), 0);

// memory_copy.wast:426
assert_return(() => call($9, "load8_u", [10_765]), 0);

// memory_copy.wast:427
assert_return(() => call($9, "load8_u", [10_964]), 0);

// memory_copy.wast:428
assert_return(() => call($9, "load8_u", [11_163]), 0);

// memory_copy.wast:429
assert_return(() => call($9, "load8_u", [11_362]), 0);

// memory_copy.wast:430
assert_return(() => call($9, "load8_u", [11_561]), 0);

// memory_copy.wast:431
assert_return(() => call($9, "load8_u", [11_760]), 0);

// memory_copy.wast:432
assert_return(() => call($9, "load8_u", [11_959]), 0);

// memory_copy.wast:433
assert_return(() => call($9, "load8_u", [12_158]), 0);

// memory_copy.wast:434
assert_return(() => call($9, "load8_u", [12_357]), 0);

// memory_copy.wast:435
assert_return(() => call($9, "load8_u", [12_556]), 0);

// memory_copy.wast:436
assert_return(() => call($9, "load8_u", [12_755]), 0);

// memory_copy.wast:437
assert_return(() => call($9, "load8_u", [12_954]), 0);

// memory_copy.wast:438
assert_return(() => call($9, "load8_u", [13_153]), 0);

// memory_copy.wast:439
assert_return(() => call($9, "load8_u", [13_352]), 0);

// memory_copy.wast:440
assert_return(() => call($9, "load8_u", [13_551]), 0);

// memory_copy.wast:441
assert_return(() => call($9, "load8_u", [13_750]), 0);

// memory_copy.wast:442
assert_return(() => call($9, "load8_u", [13_949]), 0);

// memory_copy.wast:443
assert_return(() => call($9, "load8_u", [14_148]), 0);

// memory_copy.wast:444
assert_return(() => call($9, "load8_u", [14_347]), 0);

// memory_copy.wast:445
assert_return(() => call($9, "load8_u", [14_546]), 0);

// memory_copy.wast:446
assert_return(() => call($9, "load8_u", [14_745]), 0);

// memory_copy.wast:447
assert_return(() => call($9, "load8_u", [14_944]), 0);

// memory_copy.wast:448
assert_return(() => call($9, "load8_u", [15_143]), 0);

// memory_copy.wast:449
assert_return(() => call($9, "load8_u", [15_342]), 0);

// memory_copy.wast:450
assert_return(() => call($9, "load8_u", [15_541]), 0);

// memory_copy.wast:451
assert_return(() => call($9, "load8_u", [15_740]), 0);

// memory_copy.wast:452
assert_return(() => call($9, "load8_u", [15_939]), 0);

// memory_copy.wast:453
assert_return(() => call($9, "load8_u", [16_138]), 0);

// memory_copy.wast:454
assert_return(() => call($9, "load8_u", [16_337]), 0);

// memory_copy.wast:455
assert_return(() => call($9, "load8_u", [16_536]), 0);

// memory_copy.wast:456
assert_return(() => call($9, "load8_u", [16_735]), 0);

// memory_copy.wast:457
assert_return(() => call($9, "load8_u", [16_934]), 0);

// memory_copy.wast:458
assert_return(() => call($9, "load8_u", [17_133]), 0);

// memory_copy.wast:459
assert_return(() => call($9, "load8_u", [17_332]), 0);

// memory_copy.wast:460
assert_return(() => call($9, "load8_u", [17_531]), 0);

// memory_copy.wast:461
assert_return(() => call($9, "load8_u", [17_730]), 0);

// memory_copy.wast:462
assert_return(() => call($9, "load8_u", [17_929]), 0);

// memory_copy.wast:463
assert_return(() => call($9, "load8_u", [18_128]), 0);

// memory_copy.wast:464
assert_return(() => call($9, "load8_u", [18_327]), 0);

// memory_copy.wast:465
assert_return(() => call($9, "load8_u", [18_526]), 0);

// memory_copy.wast:466
assert_return(() => call($9, "load8_u", [18_725]), 0);

// memory_copy.wast:467
assert_return(() => call($9, "load8_u", [18_924]), 0);

// memory_copy.wast:468
assert_return(() => call($9, "load8_u", [19_123]), 0);

// memory_copy.wast:469
assert_return(() => call($9, "load8_u", [19_322]), 0);

// memory_copy.wast:470
assert_return(() => call($9, "load8_u", [19_521]), 0);

// memory_copy.wast:471
assert_return(() => call($9, "load8_u", [19_720]), 0);

// memory_copy.wast:472
assert_return(() => call($9, "load8_u", [19_919]), 0);

// memory_copy.wast:473
assert_return(() => call($9, "load8_u", [20_118]), 0);

// memory_copy.wast:474
assert_return(() => call($9, "load8_u", [20_317]), 0);

// memory_copy.wast:475
assert_return(() => call($9, "load8_u", [20_516]), 0);

// memory_copy.wast:476
assert_return(() => call($9, "load8_u", [20_715]), 0);

// memory_copy.wast:477
assert_return(() => call($9, "load8_u", [20_914]), 0);

// memory_copy.wast:478
assert_return(() => call($9, "load8_u", [21_113]), 0);

// memory_copy.wast:479
assert_return(() => call($9, "load8_u", [21_312]), 0);

// memory_copy.wast:480
assert_return(() => call($9, "load8_u", [21_511]), 0);

// memory_copy.wast:481
assert_return(() => call($9, "load8_u", [21_710]), 0);

// memory_copy.wast:482
assert_return(() => call($9, "load8_u", [21_909]), 0);

// memory_copy.wast:483
assert_return(() => call($9, "load8_u", [22_108]), 0);

// memory_copy.wast:484
assert_return(() => call($9, "load8_u", [22_307]), 0);

// memory_copy.wast:485
assert_return(() => call($9, "load8_u", [22_506]), 0);

// memory_copy.wast:486
assert_return(() => call($9, "load8_u", [22_705]), 0);

// memory_copy.wast:487
assert_return(() => call($9, "load8_u", [22_904]), 0);

// memory_copy.wast:488
assert_return(() => call($9, "load8_u", [23_103]), 0);

// memory_copy.wast:489
assert_return(() => call($9, "load8_u", [23_302]), 0);

// memory_copy.wast:490
assert_return(() => call($9, "load8_u", [23_501]), 0);

// memory_copy.wast:491
assert_return(() => call($9, "load8_u", [23_700]), 0);

// memory_copy.wast:492
assert_return(() => call($9, "load8_u", [23_899]), 0);

// memory_copy.wast:493
assert_return(() => call($9, "load8_u", [24_098]), 0);

// memory_copy.wast:494
assert_return(() => call($9, "load8_u", [24_297]), 0);

// memory_copy.wast:495
assert_return(() => call($9, "load8_u", [24_496]), 0);

// memory_copy.wast:496
assert_return(() => call($9, "load8_u", [24_695]), 0);

// memory_copy.wast:497
assert_return(() => call($9, "load8_u", [24_894]), 0);

// memory_copy.wast:498
assert_return(() => call($9, "load8_u", [25_093]), 0);

// memory_copy.wast:499
assert_return(() => call($9, "load8_u", [25_292]), 0);

// memory_copy.wast:500
assert_return(() => call($9, "load8_u", [25_491]), 0);

// memory_copy.wast:501
assert_return(() => call($9, "load8_u", [25_690]), 0);

// memory_copy.wast:502
assert_return(() => call($9, "load8_u", [25_889]), 0);

// memory_copy.wast:503
assert_return(() => call($9, "load8_u", [26_088]), 0);

// memory_copy.wast:504
assert_return(() => call($9, "load8_u", [26_287]), 0);

// memory_copy.wast:505
assert_return(() => call($9, "load8_u", [26_486]), 0);

// memory_copy.wast:506
assert_return(() => call($9, "load8_u", [26_685]), 0);

// memory_copy.wast:507
assert_return(() => call($9, "load8_u", [26_884]), 0);

// memory_copy.wast:508
assert_return(() => call($9, "load8_u", [27_083]), 0);

// memory_copy.wast:509
assert_return(() => call($9, "load8_u", [27_282]), 0);

// memory_copy.wast:510
assert_return(() => call($9, "load8_u", [27_481]), 0);

// memory_copy.wast:511
assert_return(() => call($9, "load8_u", [27_680]), 0);

// memory_copy.wast:512
assert_return(() => call($9, "load8_u", [27_879]), 0);

// memory_copy.wast:513
assert_return(() => call($9, "load8_u", [28_078]), 0);

// memory_copy.wast:514
assert_return(() => call($9, "load8_u", [28_277]), 0);

// memory_copy.wast:515
assert_return(() => call($9, "load8_u", [28_476]), 0);

// memory_copy.wast:516
assert_return(() => call($9, "load8_u", [28_675]), 0);

// memory_copy.wast:517
assert_return(() => call($9, "load8_u", [28_874]), 0);

// memory_copy.wast:518
assert_return(() => call($9, "load8_u", [29_073]), 0);

// memory_copy.wast:519
assert_return(() => call($9, "load8_u", [29_272]), 0);

// memory_copy.wast:520
assert_return(() => call($9, "load8_u", [29_471]), 0);

// memory_copy.wast:521
assert_return(() => call($9, "load8_u", [29_670]), 0);

// memory_copy.wast:522
assert_return(() => call($9, "load8_u", [29_869]), 0);

// memory_copy.wast:523
assert_return(() => call($9, "load8_u", [30_068]), 0);

// memory_copy.wast:524
assert_return(() => call($9, "load8_u", [30_267]), 0);

// memory_copy.wast:525
assert_return(() => call($9, "load8_u", [30_466]), 0);

// memory_copy.wast:526
assert_return(() => call($9, "load8_u", [30_665]), 0);

// memory_copy.wast:527
assert_return(() => call($9, "load8_u", [30_864]), 0);

// memory_copy.wast:528
assert_return(() => call($9, "load8_u", [31_063]), 0);

// memory_copy.wast:529
assert_return(() => call($9, "load8_u", [31_262]), 0);

// memory_copy.wast:530
assert_return(() => call($9, "load8_u", [31_461]), 0);

// memory_copy.wast:531
assert_return(() => call($9, "load8_u", [31_660]), 0);

// memory_copy.wast:532
assert_return(() => call($9, "load8_u", [31_859]), 0);

// memory_copy.wast:533
assert_return(() => call($9, "load8_u", [32_058]), 0);

// memory_copy.wast:534
assert_return(() => call($9, "load8_u", [32_257]), 0);

// memory_copy.wast:535
assert_return(() => call($9, "load8_u", [32_456]), 0);

// memory_copy.wast:536
assert_return(() => call($9, "load8_u", [32_655]), 0);

// memory_copy.wast:537
assert_return(() => call($9, "load8_u", [32_854]), 0);

// memory_copy.wast:538
assert_return(() => call($9, "load8_u", [33_053]), 0);

// memory_copy.wast:539
assert_return(() => call($9, "load8_u", [33_252]), 0);

// memory_copy.wast:540
assert_return(() => call($9, "load8_u", [33_451]), 0);

// memory_copy.wast:541
assert_return(() => call($9, "load8_u", [33_650]), 0);

// memory_copy.wast:542
assert_return(() => call($9, "load8_u", [33_849]), 0);

// memory_copy.wast:543
assert_return(() => call($9, "load8_u", [34_048]), 0);

// memory_copy.wast:544
assert_return(() => call($9, "load8_u", [34_247]), 0);

// memory_copy.wast:545
assert_return(() => call($9, "load8_u", [34_446]), 0);

// memory_copy.wast:546
assert_return(() => call($9, "load8_u", [34_645]), 0);

// memory_copy.wast:547
assert_return(() => call($9, "load8_u", [34_844]), 0);

// memory_copy.wast:548
assert_return(() => call($9, "load8_u", [35_043]), 0);

// memory_copy.wast:549
assert_return(() => call($9, "load8_u", [35_242]), 0);

// memory_copy.wast:550
assert_return(() => call($9, "load8_u", [35_441]), 0);

// memory_copy.wast:551
assert_return(() => call($9, "load8_u", [35_640]), 0);

// memory_copy.wast:552
assert_return(() => call($9, "load8_u", [35_839]), 0);

// memory_copy.wast:553
assert_return(() => call($9, "load8_u", [36_038]), 0);

// memory_copy.wast:554
assert_return(() => call($9, "load8_u", [36_237]), 0);

// memory_copy.wast:555
assert_return(() => call($9, "load8_u", [36_436]), 0);

// memory_copy.wast:556
assert_return(() => call($9, "load8_u", [36_635]), 0);

// memory_copy.wast:557
assert_return(() => call($9, "load8_u", [36_834]), 0);

// memory_copy.wast:558
assert_return(() => call($9, "load8_u", [37_033]), 0);

// memory_copy.wast:559
assert_return(() => call($9, "load8_u", [37_232]), 0);

// memory_copy.wast:560
assert_return(() => call($9, "load8_u", [37_431]), 0);

// memory_copy.wast:561
assert_return(() => call($9, "load8_u", [37_630]), 0);

// memory_copy.wast:562
assert_return(() => call($9, "load8_u", [37_829]), 0);

// memory_copy.wast:563
assert_return(() => call($9, "load8_u", [38_028]), 0);

// memory_copy.wast:564
assert_return(() => call($9, "load8_u", [38_227]), 0);

// memory_copy.wast:565
assert_return(() => call($9, "load8_u", [38_426]), 0);

// memory_copy.wast:566
assert_return(() => call($9, "load8_u", [38_625]), 0);

// memory_copy.wast:567
assert_return(() => call($9, "load8_u", [38_824]), 0);

// memory_copy.wast:568
assert_return(() => call($9, "load8_u", [39_023]), 0);

// memory_copy.wast:569
assert_return(() => call($9, "load8_u", [39_222]), 0);

// memory_copy.wast:570
assert_return(() => call($9, "load8_u", [39_421]), 0);

// memory_copy.wast:571
assert_return(() => call($9, "load8_u", [39_620]), 0);

// memory_copy.wast:572
assert_return(() => call($9, "load8_u", [39_819]), 0);

// memory_copy.wast:573
assert_return(() => call($9, "load8_u", [40_018]), 0);

// memory_copy.wast:574
assert_return(() => call($9, "load8_u", [40_217]), 0);

// memory_copy.wast:575
assert_return(() => call($9, "load8_u", [40_416]), 0);

// memory_copy.wast:576
assert_return(() => call($9, "load8_u", [40_615]), 0);

// memory_copy.wast:577
assert_return(() => call($9, "load8_u", [40_814]), 0);

// memory_copy.wast:578
assert_return(() => call($9, "load8_u", [41_013]), 0);

// memory_copy.wast:579
assert_return(() => call($9, "load8_u", [41_212]), 0);

// memory_copy.wast:580
assert_return(() => call($9, "load8_u", [41_411]), 0);

// memory_copy.wast:581
assert_return(() => call($9, "load8_u", [41_610]), 0);

// memory_copy.wast:582
assert_return(() => call($9, "load8_u", [41_809]), 0);

// memory_copy.wast:583
assert_return(() => call($9, "load8_u", [42_008]), 0);

// memory_copy.wast:584
assert_return(() => call($9, "load8_u", [42_207]), 0);

// memory_copy.wast:585
assert_return(() => call($9, "load8_u", [42_406]), 0);

// memory_copy.wast:586
assert_return(() => call($9, "load8_u", [42_605]), 0);

// memory_copy.wast:587
assert_return(() => call($9, "load8_u", [42_804]), 0);

// memory_copy.wast:588
assert_return(() => call($9, "load8_u", [43_003]), 0);

// memory_copy.wast:589
assert_return(() => call($9, "load8_u", [43_202]), 0);

// memory_copy.wast:590
assert_return(() => call($9, "load8_u", [43_401]), 0);

// memory_copy.wast:591
assert_return(() => call($9, "load8_u", [43_600]), 0);

// memory_copy.wast:592
assert_return(() => call($9, "load8_u", [43_799]), 0);

// memory_copy.wast:593
assert_return(() => call($9, "load8_u", [43_998]), 0);

// memory_copy.wast:594
assert_return(() => call($9, "load8_u", [44_197]), 0);

// memory_copy.wast:595
assert_return(() => call($9, "load8_u", [44_396]), 0);

// memory_copy.wast:596
assert_return(() => call($9, "load8_u", [44_595]), 0);

// memory_copy.wast:597
assert_return(() => call($9, "load8_u", [44_794]), 0);

// memory_copy.wast:598
assert_return(() => call($9, "load8_u", [44_993]), 0);

// memory_copy.wast:599
assert_return(() => call($9, "load8_u", [45_192]), 0);

// memory_copy.wast:600
assert_return(() => call($9, "load8_u", [45_391]), 0);

// memory_copy.wast:601
assert_return(() => call($9, "load8_u", [45_590]), 0);

// memory_copy.wast:602
assert_return(() => call($9, "load8_u", [45_789]), 0);

// memory_copy.wast:603
assert_return(() => call($9, "load8_u", [45_988]), 0);

// memory_copy.wast:604
assert_return(() => call($9, "load8_u", [46_187]), 0);

// memory_copy.wast:605
assert_return(() => call($9, "load8_u", [46_386]), 0);

// memory_copy.wast:606
assert_return(() => call($9, "load8_u", [46_585]), 0);

// memory_copy.wast:607
assert_return(() => call($9, "load8_u", [46_784]), 0);

// memory_copy.wast:608
assert_return(() => call($9, "load8_u", [46_983]), 0);

// memory_copy.wast:609
assert_return(() => call($9, "load8_u", [47_182]), 0);

// memory_copy.wast:610
assert_return(() => call($9, "load8_u", [47_381]), 0);

// memory_copy.wast:611
assert_return(() => call($9, "load8_u", [47_580]), 0);

// memory_copy.wast:612
assert_return(() => call($9, "load8_u", [47_779]), 0);

// memory_copy.wast:613
assert_return(() => call($9, "load8_u", [47_978]), 0);

// memory_copy.wast:614
assert_return(() => call($9, "load8_u", [48_177]), 0);

// memory_copy.wast:615
assert_return(() => call($9, "load8_u", [48_376]), 0);

// memory_copy.wast:616
assert_return(() => call($9, "load8_u", [48_575]), 0);

// memory_copy.wast:617
assert_return(() => call($9, "load8_u", [48_774]), 0);

// memory_copy.wast:618
assert_return(() => call($9, "load8_u", [48_973]), 0);

// memory_copy.wast:619
assert_return(() => call($9, "load8_u", [49_172]), 0);

// memory_copy.wast:620
assert_return(() => call($9, "load8_u", [49_371]), 0);

// memory_copy.wast:621
assert_return(() => call($9, "load8_u", [49_570]), 0);

// memory_copy.wast:622
assert_return(() => call($9, "load8_u", [49_769]), 0);

// memory_copy.wast:623
assert_return(() => call($9, "load8_u", [49_968]), 0);

// memory_copy.wast:624
assert_return(() => call($9, "load8_u", [50_167]), 0);

// memory_copy.wast:625
assert_return(() => call($9, "load8_u", [50_366]), 0);

// memory_copy.wast:626
assert_return(() => call($9, "load8_u", [50_565]), 0);

// memory_copy.wast:627
assert_return(() => call($9, "load8_u", [50_764]), 0);

// memory_copy.wast:628
assert_return(() => call($9, "load8_u", [50_963]), 0);

// memory_copy.wast:629
assert_return(() => call($9, "load8_u", [51_162]), 0);

// memory_copy.wast:630
assert_return(() => call($9, "load8_u", [51_361]), 0);

// memory_copy.wast:631
assert_return(() => call($9, "load8_u", [51_560]), 0);

// memory_copy.wast:632
assert_return(() => call($9, "load8_u", [51_759]), 0);

// memory_copy.wast:633
assert_return(() => call($9, "load8_u", [51_958]), 0);

// memory_copy.wast:634
assert_return(() => call($9, "load8_u", [52_157]), 0);

// memory_copy.wast:635
assert_return(() => call($9, "load8_u", [52_356]), 0);

// memory_copy.wast:636
assert_return(() => call($9, "load8_u", [52_555]), 0);

// memory_copy.wast:637
assert_return(() => call($9, "load8_u", [52_754]), 0);

// memory_copy.wast:638
assert_return(() => call($9, "load8_u", [52_953]), 0);

// memory_copy.wast:639
assert_return(() => call($9, "load8_u", [53_152]), 0);

// memory_copy.wast:640
assert_return(() => call($9, "load8_u", [53_351]), 0);

// memory_copy.wast:641
assert_return(() => call($9, "load8_u", [53_550]), 0);

// memory_copy.wast:642
assert_return(() => call($9, "load8_u", [53_749]), 0);

// memory_copy.wast:643
assert_return(() => call($9, "load8_u", [53_948]), 0);

// memory_copy.wast:644
assert_return(() => call($9, "load8_u", [54_147]), 0);

// memory_copy.wast:645
assert_return(() => call($9, "load8_u", [54_346]), 0);

// memory_copy.wast:646
assert_return(() => call($9, "load8_u", [54_545]), 0);

// memory_copy.wast:647
assert_return(() => call($9, "load8_u", [54_744]), 0);

// memory_copy.wast:648
assert_return(() => call($9, "load8_u", [54_943]), 0);

// memory_copy.wast:649
assert_return(() => call($9, "load8_u", [55_142]), 0);

// memory_copy.wast:650
assert_return(() => call($9, "load8_u", [55_341]), 0);

// memory_copy.wast:651
assert_return(() => call($9, "load8_u", [55_540]), 0);

// memory_copy.wast:652
assert_return(() => call($9, "load8_u", [55_739]), 0);

// memory_copy.wast:653
assert_return(() => call($9, "load8_u", [55_938]), 0);

// memory_copy.wast:654
assert_return(() => call($9, "load8_u", [56_137]), 0);

// memory_copy.wast:655
assert_return(() => call($9, "load8_u", [56_336]), 0);

// memory_copy.wast:656
assert_return(() => call($9, "load8_u", [56_535]), 0);

// memory_copy.wast:657
assert_return(() => call($9, "load8_u", [56_734]), 0);

// memory_copy.wast:658
assert_return(() => call($9, "load8_u", [56_933]), 0);

// memory_copy.wast:659
assert_return(() => call($9, "load8_u", [57_132]), 0);

// memory_copy.wast:660
assert_return(() => call($9, "load8_u", [57_331]), 0);

// memory_copy.wast:661
assert_return(() => call($9, "load8_u", [57_530]), 0);

// memory_copy.wast:662
assert_return(() => call($9, "load8_u", [57_729]), 0);

// memory_copy.wast:663
assert_return(() => call($9, "load8_u", [57_928]), 0);

// memory_copy.wast:664
assert_return(() => call($9, "load8_u", [58_127]), 0);

// memory_copy.wast:665
assert_return(() => call($9, "load8_u", [58_326]), 0);

// memory_copy.wast:666
assert_return(() => call($9, "load8_u", [58_525]), 0);

// memory_copy.wast:667
assert_return(() => call($9, "load8_u", [58_724]), 0);

// memory_copy.wast:668
assert_return(() => call($9, "load8_u", [58_923]), 0);

// memory_copy.wast:669
assert_return(() => call($9, "load8_u", [59_122]), 0);

// memory_copy.wast:670
assert_return(() => call($9, "load8_u", [59_321]), 0);

// memory_copy.wast:671
assert_return(() => call($9, "load8_u", [59_520]), 0);

// memory_copy.wast:672
assert_return(() => call($9, "load8_u", [59_719]), 0);

// memory_copy.wast:673
assert_return(() => call($9, "load8_u", [59_918]), 0);

// memory_copy.wast:674
assert_return(() => call($9, "load8_u", [60_117]), 0);

// memory_copy.wast:675
assert_return(() => call($9, "load8_u", [60_316]), 0);

// memory_copy.wast:676
assert_return(() => call($9, "load8_u", [60_515]), 0);

// memory_copy.wast:677
assert_return(() => call($9, "load8_u", [60_714]), 0);

// memory_copy.wast:678
assert_return(() => call($9, "load8_u", [60_913]), 0);

// memory_copy.wast:679
assert_return(() => call($9, "load8_u", [61_112]), 0);

// memory_copy.wast:680
assert_return(() => call($9, "load8_u", [61_311]), 0);

// memory_copy.wast:681
assert_return(() => call($9, "load8_u", [61_510]), 0);

// memory_copy.wast:682
assert_return(() => call($9, "load8_u", [61_709]), 0);

// memory_copy.wast:683
assert_return(() => call($9, "load8_u", [61_908]), 0);

// memory_copy.wast:684
assert_return(() => call($9, "load8_u", [62_107]), 0);

// memory_copy.wast:685
assert_return(() => call($9, "load8_u", [62_306]), 0);

// memory_copy.wast:686
assert_return(() => call($9, "load8_u", [62_505]), 0);

// memory_copy.wast:687
assert_return(() => call($9, "load8_u", [62_704]), 0);

// memory_copy.wast:688
assert_return(() => call($9, "load8_u", [62_903]), 0);

// memory_copy.wast:689
assert_return(() => call($9, "load8_u", [63_102]), 0);

// memory_copy.wast:690
assert_return(() => call($9, "load8_u", [63_301]), 0);

// memory_copy.wast:691
assert_return(() => call($9, "load8_u", [63_500]), 0);

// memory_copy.wast:692
assert_return(() => call($9, "load8_u", [63_699]), 0);

// memory_copy.wast:693
assert_return(() => call($9, "load8_u", [63_898]), 0);

// memory_copy.wast:694
assert_return(() => call($9, "load8_u", [64_097]), 0);

// memory_copy.wast:695
assert_return(() => call($9, "load8_u", [64_296]), 0);

// memory_copy.wast:696
assert_return(() => call($9, "load8_u", [64_495]), 0);

// memory_copy.wast:697
assert_return(() => call($9, "load8_u", [64_694]), 0);

// memory_copy.wast:698
assert_return(() => call($9, "load8_u", [64_893]), 0);

// memory_copy.wast:699
assert_return(() => call($9, "load8_u", [65_092]), 0);

// memory_copy.wast:700
assert_return(() => call($9, "load8_u", [65_291]), 0);

// memory_copy.wast:701
assert_return(() => call($9, "load8_u", [65_490]), 0);

// memory_copy.wast:703
let $10 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9b\x80\x80\x80\x00\x01\x00\x41\x00\x0b\x15\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\x14");

// memory_copy.wast:711
assert_trap(() => call($10, "run", [65_515, 0, 39]));

// memory_copy.wast:714
assert_return(() => call($10, "load8_u", [0]), 0);

// memory_copy.wast:715
assert_return(() => call($10, "load8_u", [1]), 1);

// memory_copy.wast:716
assert_return(() => call($10, "load8_u", [2]), 2);

// memory_copy.wast:717
assert_return(() => call($10, "load8_u", [3]), 3);

// memory_copy.wast:718
assert_return(() => call($10, "load8_u", [4]), 4);

// memory_copy.wast:719
assert_return(() => call($10, "load8_u", [5]), 5);

// memory_copy.wast:720
assert_return(() => call($10, "load8_u", [6]), 6);

// memory_copy.wast:721
assert_return(() => call($10, "load8_u", [7]), 7);

// memory_copy.wast:722
assert_return(() => call($10, "load8_u", [8]), 8);

// memory_copy.wast:723
assert_return(() => call($10, "load8_u", [9]), 9);

// memory_copy.wast:724
assert_return(() => call($10, "load8_u", [10]), 10);

// memory_copy.wast:725
assert_return(() => call($10, "load8_u", [11]), 11);

// memory_copy.wast:726
assert_return(() => call($10, "load8_u", [12]), 12);

// memory_copy.wast:727
assert_return(() => call($10, "load8_u", [13]), 13);

// memory_copy.wast:728
assert_return(() => call($10, "load8_u", [14]), 14);

// memory_copy.wast:729
assert_return(() => call($10, "load8_u", [15]), 15);

// memory_copy.wast:730
assert_return(() => call($10, "load8_u", [16]), 16);

// memory_copy.wast:731
assert_return(() => call($10, "load8_u", [17]), 17);

// memory_copy.wast:732
assert_return(() => call($10, "load8_u", [18]), 18);

// memory_copy.wast:733
assert_return(() => call($10, "load8_u", [19]), 19);

// memory_copy.wast:734
assert_return(() => call($10, "load8_u", [20]), 20);

// memory_copy.wast:735
assert_return(() => call($10, "load8_u", [219]), 0);

// memory_copy.wast:736
assert_return(() => call($10, "load8_u", [418]), 0);

// memory_copy.wast:737
assert_return(() => call($10, "load8_u", [617]), 0);

// memory_copy.wast:738
assert_return(() => call($10, "load8_u", [816]), 0);

// memory_copy.wast:739
assert_return(() => call($10, "load8_u", [1_015]), 0);

// memory_copy.wast:740
assert_return(() => call($10, "load8_u", [1_214]), 0);

// memory_copy.wast:741
assert_return(() => call($10, "load8_u", [1_413]), 0);

// memory_copy.wast:742
assert_return(() => call($10, "load8_u", [1_612]), 0);

// memory_copy.wast:743
assert_return(() => call($10, "load8_u", [1_811]), 0);

// memory_copy.wast:744
assert_return(() => call($10, "load8_u", [2_010]), 0);

// memory_copy.wast:745
assert_return(() => call($10, "load8_u", [2_209]), 0);

// memory_copy.wast:746
assert_return(() => call($10, "load8_u", [2_408]), 0);

// memory_copy.wast:747
assert_return(() => call($10, "load8_u", [2_607]), 0);

// memory_copy.wast:748
assert_return(() => call($10, "load8_u", [2_806]), 0);

// memory_copy.wast:749
assert_return(() => call($10, "load8_u", [3_005]), 0);

// memory_copy.wast:750
assert_return(() => call($10, "load8_u", [3_204]), 0);

// memory_copy.wast:751
assert_return(() => call($10, "load8_u", [3_403]), 0);

// memory_copy.wast:752
assert_return(() => call($10, "load8_u", [3_602]), 0);

// memory_copy.wast:753
assert_return(() => call($10, "load8_u", [3_801]), 0);

// memory_copy.wast:754
assert_return(() => call($10, "load8_u", [4_000]), 0);

// memory_copy.wast:755
assert_return(() => call($10, "load8_u", [4_199]), 0);

// memory_copy.wast:756
assert_return(() => call($10, "load8_u", [4_398]), 0);

// memory_copy.wast:757
assert_return(() => call($10, "load8_u", [4_597]), 0);

// memory_copy.wast:758
assert_return(() => call($10, "load8_u", [4_796]), 0);

// memory_copy.wast:759
assert_return(() => call($10, "load8_u", [4_995]), 0);

// memory_copy.wast:760
assert_return(() => call($10, "load8_u", [5_194]), 0);

// memory_copy.wast:761
assert_return(() => call($10, "load8_u", [5_393]), 0);

// memory_copy.wast:762
assert_return(() => call($10, "load8_u", [5_592]), 0);

// memory_copy.wast:763
assert_return(() => call($10, "load8_u", [5_791]), 0);

// memory_copy.wast:764
assert_return(() => call($10, "load8_u", [5_990]), 0);

// memory_copy.wast:765
assert_return(() => call($10, "load8_u", [6_189]), 0);

// memory_copy.wast:766
assert_return(() => call($10, "load8_u", [6_388]), 0);

// memory_copy.wast:767
assert_return(() => call($10, "load8_u", [6_587]), 0);

// memory_copy.wast:768
assert_return(() => call($10, "load8_u", [6_786]), 0);

// memory_copy.wast:769
assert_return(() => call($10, "load8_u", [6_985]), 0);

// memory_copy.wast:770
assert_return(() => call($10, "load8_u", [7_184]), 0);

// memory_copy.wast:771
assert_return(() => call($10, "load8_u", [7_383]), 0);

// memory_copy.wast:772
assert_return(() => call($10, "load8_u", [7_582]), 0);

// memory_copy.wast:773
assert_return(() => call($10, "load8_u", [7_781]), 0);

// memory_copy.wast:774
assert_return(() => call($10, "load8_u", [7_980]), 0);

// memory_copy.wast:775
assert_return(() => call($10, "load8_u", [8_179]), 0);

// memory_copy.wast:776
assert_return(() => call($10, "load8_u", [8_378]), 0);

// memory_copy.wast:777
assert_return(() => call($10, "load8_u", [8_577]), 0);

// memory_copy.wast:778
assert_return(() => call($10, "load8_u", [8_776]), 0);

// memory_copy.wast:779
assert_return(() => call($10, "load8_u", [8_975]), 0);

// memory_copy.wast:780
assert_return(() => call($10, "load8_u", [9_174]), 0);

// memory_copy.wast:781
assert_return(() => call($10, "load8_u", [9_373]), 0);

// memory_copy.wast:782
assert_return(() => call($10, "load8_u", [9_572]), 0);

// memory_copy.wast:783
assert_return(() => call($10, "load8_u", [9_771]), 0);

// memory_copy.wast:784
assert_return(() => call($10, "load8_u", [9_970]), 0);

// memory_copy.wast:785
assert_return(() => call($10, "load8_u", [10_169]), 0);

// memory_copy.wast:786
assert_return(() => call($10, "load8_u", [10_368]), 0);

// memory_copy.wast:787
assert_return(() => call($10, "load8_u", [10_567]), 0);

// memory_copy.wast:788
assert_return(() => call($10, "load8_u", [10_766]), 0);

// memory_copy.wast:789
assert_return(() => call($10, "load8_u", [10_965]), 0);

// memory_copy.wast:790
assert_return(() => call($10, "load8_u", [11_164]), 0);

// memory_copy.wast:791
assert_return(() => call($10, "load8_u", [11_363]), 0);

// memory_copy.wast:792
assert_return(() => call($10, "load8_u", [11_562]), 0);

// memory_copy.wast:793
assert_return(() => call($10, "load8_u", [11_761]), 0);

// memory_copy.wast:794
assert_return(() => call($10, "load8_u", [11_960]), 0);

// memory_copy.wast:795
assert_return(() => call($10, "load8_u", [12_159]), 0);

// memory_copy.wast:796
assert_return(() => call($10, "load8_u", [12_358]), 0);

// memory_copy.wast:797
assert_return(() => call($10, "load8_u", [12_557]), 0);

// memory_copy.wast:798
assert_return(() => call($10, "load8_u", [12_756]), 0);

// memory_copy.wast:799
assert_return(() => call($10, "load8_u", [12_955]), 0);

// memory_copy.wast:800
assert_return(() => call($10, "load8_u", [13_154]), 0);

// memory_copy.wast:801
assert_return(() => call($10, "load8_u", [13_353]), 0);

// memory_copy.wast:802
assert_return(() => call($10, "load8_u", [13_552]), 0);

// memory_copy.wast:803
assert_return(() => call($10, "load8_u", [13_751]), 0);

// memory_copy.wast:804
assert_return(() => call($10, "load8_u", [13_950]), 0);

// memory_copy.wast:805
assert_return(() => call($10, "load8_u", [14_149]), 0);

// memory_copy.wast:806
assert_return(() => call($10, "load8_u", [14_348]), 0);

// memory_copy.wast:807
assert_return(() => call($10, "load8_u", [14_547]), 0);

// memory_copy.wast:808
assert_return(() => call($10, "load8_u", [14_746]), 0);

// memory_copy.wast:809
assert_return(() => call($10, "load8_u", [14_945]), 0);

// memory_copy.wast:810
assert_return(() => call($10, "load8_u", [15_144]), 0);

// memory_copy.wast:811
assert_return(() => call($10, "load8_u", [15_343]), 0);

// memory_copy.wast:812
assert_return(() => call($10, "load8_u", [15_542]), 0);

// memory_copy.wast:813
assert_return(() => call($10, "load8_u", [15_741]), 0);

// memory_copy.wast:814
assert_return(() => call($10, "load8_u", [15_940]), 0);

// memory_copy.wast:815
assert_return(() => call($10, "load8_u", [16_139]), 0);

// memory_copy.wast:816
assert_return(() => call($10, "load8_u", [16_338]), 0);

// memory_copy.wast:817
assert_return(() => call($10, "load8_u", [16_537]), 0);

// memory_copy.wast:818
assert_return(() => call($10, "load8_u", [16_736]), 0);

// memory_copy.wast:819
assert_return(() => call($10, "load8_u", [16_935]), 0);

// memory_copy.wast:820
assert_return(() => call($10, "load8_u", [17_134]), 0);

// memory_copy.wast:821
assert_return(() => call($10, "load8_u", [17_333]), 0);

// memory_copy.wast:822
assert_return(() => call($10, "load8_u", [17_532]), 0);

// memory_copy.wast:823
assert_return(() => call($10, "load8_u", [17_731]), 0);

// memory_copy.wast:824
assert_return(() => call($10, "load8_u", [17_930]), 0);

// memory_copy.wast:825
assert_return(() => call($10, "load8_u", [18_129]), 0);

// memory_copy.wast:826
assert_return(() => call($10, "load8_u", [18_328]), 0);

// memory_copy.wast:827
assert_return(() => call($10, "load8_u", [18_527]), 0);

// memory_copy.wast:828
assert_return(() => call($10, "load8_u", [18_726]), 0);

// memory_copy.wast:829
assert_return(() => call($10, "load8_u", [18_925]), 0);

// memory_copy.wast:830
assert_return(() => call($10, "load8_u", [19_124]), 0);

// memory_copy.wast:831
assert_return(() => call($10, "load8_u", [19_323]), 0);

// memory_copy.wast:832
assert_return(() => call($10, "load8_u", [19_522]), 0);

// memory_copy.wast:833
assert_return(() => call($10, "load8_u", [19_721]), 0);

// memory_copy.wast:834
assert_return(() => call($10, "load8_u", [19_920]), 0);

// memory_copy.wast:835
assert_return(() => call($10, "load8_u", [20_119]), 0);

// memory_copy.wast:836
assert_return(() => call($10, "load8_u", [20_318]), 0);

// memory_copy.wast:837
assert_return(() => call($10, "load8_u", [20_517]), 0);

// memory_copy.wast:838
assert_return(() => call($10, "load8_u", [20_716]), 0);

// memory_copy.wast:839
assert_return(() => call($10, "load8_u", [20_915]), 0);

// memory_copy.wast:840
assert_return(() => call($10, "load8_u", [21_114]), 0);

// memory_copy.wast:841
assert_return(() => call($10, "load8_u", [21_313]), 0);

// memory_copy.wast:842
assert_return(() => call($10, "load8_u", [21_512]), 0);

// memory_copy.wast:843
assert_return(() => call($10, "load8_u", [21_711]), 0);

// memory_copy.wast:844
assert_return(() => call($10, "load8_u", [21_910]), 0);

// memory_copy.wast:845
assert_return(() => call($10, "load8_u", [22_109]), 0);

// memory_copy.wast:846
assert_return(() => call($10, "load8_u", [22_308]), 0);

// memory_copy.wast:847
assert_return(() => call($10, "load8_u", [22_507]), 0);

// memory_copy.wast:848
assert_return(() => call($10, "load8_u", [22_706]), 0);

// memory_copy.wast:849
assert_return(() => call($10, "load8_u", [22_905]), 0);

// memory_copy.wast:850
assert_return(() => call($10, "load8_u", [23_104]), 0);

// memory_copy.wast:851
assert_return(() => call($10, "load8_u", [23_303]), 0);

// memory_copy.wast:852
assert_return(() => call($10, "load8_u", [23_502]), 0);

// memory_copy.wast:853
assert_return(() => call($10, "load8_u", [23_701]), 0);

// memory_copy.wast:854
assert_return(() => call($10, "load8_u", [23_900]), 0);

// memory_copy.wast:855
assert_return(() => call($10, "load8_u", [24_099]), 0);

// memory_copy.wast:856
assert_return(() => call($10, "load8_u", [24_298]), 0);

// memory_copy.wast:857
assert_return(() => call($10, "load8_u", [24_497]), 0);

// memory_copy.wast:858
assert_return(() => call($10, "load8_u", [24_696]), 0);

// memory_copy.wast:859
assert_return(() => call($10, "load8_u", [24_895]), 0);

// memory_copy.wast:860
assert_return(() => call($10, "load8_u", [25_094]), 0);

// memory_copy.wast:861
assert_return(() => call($10, "load8_u", [25_293]), 0);

// memory_copy.wast:862
assert_return(() => call($10, "load8_u", [25_492]), 0);

// memory_copy.wast:863
assert_return(() => call($10, "load8_u", [25_691]), 0);

// memory_copy.wast:864
assert_return(() => call($10, "load8_u", [25_890]), 0);

// memory_copy.wast:865
assert_return(() => call($10, "load8_u", [26_089]), 0);

// memory_copy.wast:866
assert_return(() => call($10, "load8_u", [26_288]), 0);

// memory_copy.wast:867
assert_return(() => call($10, "load8_u", [26_487]), 0);

// memory_copy.wast:868
assert_return(() => call($10, "load8_u", [26_686]), 0);

// memory_copy.wast:869
assert_return(() => call($10, "load8_u", [26_885]), 0);

// memory_copy.wast:870
assert_return(() => call($10, "load8_u", [27_084]), 0);

// memory_copy.wast:871
assert_return(() => call($10, "load8_u", [27_283]), 0);

// memory_copy.wast:872
assert_return(() => call($10, "load8_u", [27_482]), 0);

// memory_copy.wast:873
assert_return(() => call($10, "load8_u", [27_681]), 0);

// memory_copy.wast:874
assert_return(() => call($10, "load8_u", [27_880]), 0);

// memory_copy.wast:875
assert_return(() => call($10, "load8_u", [28_079]), 0);

// memory_copy.wast:876
assert_return(() => call($10, "load8_u", [28_278]), 0);

// memory_copy.wast:877
assert_return(() => call($10, "load8_u", [28_477]), 0);

// memory_copy.wast:878
assert_return(() => call($10, "load8_u", [28_676]), 0);

// memory_copy.wast:879
assert_return(() => call($10, "load8_u", [28_875]), 0);

// memory_copy.wast:880
assert_return(() => call($10, "load8_u", [29_074]), 0);

// memory_copy.wast:881
assert_return(() => call($10, "load8_u", [29_273]), 0);

// memory_copy.wast:882
assert_return(() => call($10, "load8_u", [29_472]), 0);

// memory_copy.wast:883
assert_return(() => call($10, "load8_u", [29_671]), 0);

// memory_copy.wast:884
assert_return(() => call($10, "load8_u", [29_870]), 0);

// memory_copy.wast:885
assert_return(() => call($10, "load8_u", [30_069]), 0);

// memory_copy.wast:886
assert_return(() => call($10, "load8_u", [30_268]), 0);

// memory_copy.wast:887
assert_return(() => call($10, "load8_u", [30_467]), 0);

// memory_copy.wast:888
assert_return(() => call($10, "load8_u", [30_666]), 0);

// memory_copy.wast:889
assert_return(() => call($10, "load8_u", [30_865]), 0);

// memory_copy.wast:890
assert_return(() => call($10, "load8_u", [31_064]), 0);

// memory_copy.wast:891
assert_return(() => call($10, "load8_u", [31_263]), 0);

// memory_copy.wast:892
assert_return(() => call($10, "load8_u", [31_462]), 0);

// memory_copy.wast:893
assert_return(() => call($10, "load8_u", [31_661]), 0);

// memory_copy.wast:894
assert_return(() => call($10, "load8_u", [31_860]), 0);

// memory_copy.wast:895
assert_return(() => call($10, "load8_u", [32_059]), 0);

// memory_copy.wast:896
assert_return(() => call($10, "load8_u", [32_258]), 0);

// memory_copy.wast:897
assert_return(() => call($10, "load8_u", [32_457]), 0);

// memory_copy.wast:898
assert_return(() => call($10, "load8_u", [32_656]), 0);

// memory_copy.wast:899
assert_return(() => call($10, "load8_u", [32_855]), 0);

// memory_copy.wast:900
assert_return(() => call($10, "load8_u", [33_054]), 0);

// memory_copy.wast:901
assert_return(() => call($10, "load8_u", [33_253]), 0);

// memory_copy.wast:902
assert_return(() => call($10, "load8_u", [33_452]), 0);

// memory_copy.wast:903
assert_return(() => call($10, "load8_u", [33_651]), 0);

// memory_copy.wast:904
assert_return(() => call($10, "load8_u", [33_850]), 0);

// memory_copy.wast:905
assert_return(() => call($10, "load8_u", [34_049]), 0);

// memory_copy.wast:906
assert_return(() => call($10, "load8_u", [34_248]), 0);

// memory_copy.wast:907
assert_return(() => call($10, "load8_u", [34_447]), 0);

// memory_copy.wast:908
assert_return(() => call($10, "load8_u", [34_646]), 0);

// memory_copy.wast:909
assert_return(() => call($10, "load8_u", [34_845]), 0);

// memory_copy.wast:910
assert_return(() => call($10, "load8_u", [35_044]), 0);

// memory_copy.wast:911
assert_return(() => call($10, "load8_u", [35_243]), 0);

// memory_copy.wast:912
assert_return(() => call($10, "load8_u", [35_442]), 0);

// memory_copy.wast:913
assert_return(() => call($10, "load8_u", [35_641]), 0);

// memory_copy.wast:914
assert_return(() => call($10, "load8_u", [35_840]), 0);

// memory_copy.wast:915
assert_return(() => call($10, "load8_u", [36_039]), 0);

// memory_copy.wast:916
assert_return(() => call($10, "load8_u", [36_238]), 0);

// memory_copy.wast:917
assert_return(() => call($10, "load8_u", [36_437]), 0);

// memory_copy.wast:918
assert_return(() => call($10, "load8_u", [36_636]), 0);

// memory_copy.wast:919
assert_return(() => call($10, "load8_u", [36_835]), 0);

// memory_copy.wast:920
assert_return(() => call($10, "load8_u", [37_034]), 0);

// memory_copy.wast:921
assert_return(() => call($10, "load8_u", [37_233]), 0);

// memory_copy.wast:922
assert_return(() => call($10, "load8_u", [37_432]), 0);

// memory_copy.wast:923
assert_return(() => call($10, "load8_u", [37_631]), 0);

// memory_copy.wast:924
assert_return(() => call($10, "load8_u", [37_830]), 0);

// memory_copy.wast:925
assert_return(() => call($10, "load8_u", [38_029]), 0);

// memory_copy.wast:926
assert_return(() => call($10, "load8_u", [38_228]), 0);

// memory_copy.wast:927
assert_return(() => call($10, "load8_u", [38_427]), 0);

// memory_copy.wast:928
assert_return(() => call($10, "load8_u", [38_626]), 0);

// memory_copy.wast:929
assert_return(() => call($10, "load8_u", [38_825]), 0);

// memory_copy.wast:930
assert_return(() => call($10, "load8_u", [39_024]), 0);

// memory_copy.wast:931
assert_return(() => call($10, "load8_u", [39_223]), 0);

// memory_copy.wast:932
assert_return(() => call($10, "load8_u", [39_422]), 0);

// memory_copy.wast:933
assert_return(() => call($10, "load8_u", [39_621]), 0);

// memory_copy.wast:934
assert_return(() => call($10, "load8_u", [39_820]), 0);

// memory_copy.wast:935
assert_return(() => call($10, "load8_u", [40_019]), 0);

// memory_copy.wast:936
assert_return(() => call($10, "load8_u", [40_218]), 0);

// memory_copy.wast:937
assert_return(() => call($10, "load8_u", [40_417]), 0);

// memory_copy.wast:938
assert_return(() => call($10, "load8_u", [40_616]), 0);

// memory_copy.wast:939
assert_return(() => call($10, "load8_u", [40_815]), 0);

// memory_copy.wast:940
assert_return(() => call($10, "load8_u", [41_014]), 0);

// memory_copy.wast:941
assert_return(() => call($10, "load8_u", [41_213]), 0);

// memory_copy.wast:942
assert_return(() => call($10, "load8_u", [41_412]), 0);

// memory_copy.wast:943
assert_return(() => call($10, "load8_u", [41_611]), 0);

// memory_copy.wast:944
assert_return(() => call($10, "load8_u", [41_810]), 0);

// memory_copy.wast:945
assert_return(() => call($10, "load8_u", [42_009]), 0);

// memory_copy.wast:946
assert_return(() => call($10, "load8_u", [42_208]), 0);

// memory_copy.wast:947
assert_return(() => call($10, "load8_u", [42_407]), 0);

// memory_copy.wast:948
assert_return(() => call($10, "load8_u", [42_606]), 0);

// memory_copy.wast:949
assert_return(() => call($10, "load8_u", [42_805]), 0);

// memory_copy.wast:950
assert_return(() => call($10, "load8_u", [43_004]), 0);

// memory_copy.wast:951
assert_return(() => call($10, "load8_u", [43_203]), 0);

// memory_copy.wast:952
assert_return(() => call($10, "load8_u", [43_402]), 0);

// memory_copy.wast:953
assert_return(() => call($10, "load8_u", [43_601]), 0);

// memory_copy.wast:954
assert_return(() => call($10, "load8_u", [43_800]), 0);

// memory_copy.wast:955
assert_return(() => call($10, "load8_u", [43_999]), 0);

// memory_copy.wast:956
assert_return(() => call($10, "load8_u", [44_198]), 0);

// memory_copy.wast:957
assert_return(() => call($10, "load8_u", [44_397]), 0);

// memory_copy.wast:958
assert_return(() => call($10, "load8_u", [44_596]), 0);

// memory_copy.wast:959
assert_return(() => call($10, "load8_u", [44_795]), 0);

// memory_copy.wast:960
assert_return(() => call($10, "load8_u", [44_994]), 0);

// memory_copy.wast:961
assert_return(() => call($10, "load8_u", [45_193]), 0);

// memory_copy.wast:962
assert_return(() => call($10, "load8_u", [45_392]), 0);

// memory_copy.wast:963
assert_return(() => call($10, "load8_u", [45_591]), 0);

// memory_copy.wast:964
assert_return(() => call($10, "load8_u", [45_790]), 0);

// memory_copy.wast:965
assert_return(() => call($10, "load8_u", [45_989]), 0);

// memory_copy.wast:966
assert_return(() => call($10, "load8_u", [46_188]), 0);

// memory_copy.wast:967
assert_return(() => call($10, "load8_u", [46_387]), 0);

// memory_copy.wast:968
assert_return(() => call($10, "load8_u", [46_586]), 0);

// memory_copy.wast:969
assert_return(() => call($10, "load8_u", [46_785]), 0);

// memory_copy.wast:970
assert_return(() => call($10, "load8_u", [46_984]), 0);

// memory_copy.wast:971
assert_return(() => call($10, "load8_u", [47_183]), 0);

// memory_copy.wast:972
assert_return(() => call($10, "load8_u", [47_382]), 0);

// memory_copy.wast:973
assert_return(() => call($10, "load8_u", [47_581]), 0);

// memory_copy.wast:974
assert_return(() => call($10, "load8_u", [47_780]), 0);

// memory_copy.wast:975
assert_return(() => call($10, "load8_u", [47_979]), 0);

// memory_copy.wast:976
assert_return(() => call($10, "load8_u", [48_178]), 0);

// memory_copy.wast:977
assert_return(() => call($10, "load8_u", [48_377]), 0);

// memory_copy.wast:978
assert_return(() => call($10, "load8_u", [48_576]), 0);

// memory_copy.wast:979
assert_return(() => call($10, "load8_u", [48_775]), 0);

// memory_copy.wast:980
assert_return(() => call($10, "load8_u", [48_974]), 0);

// memory_copy.wast:981
assert_return(() => call($10, "load8_u", [49_173]), 0);

// memory_copy.wast:982
assert_return(() => call($10, "load8_u", [49_372]), 0);

// memory_copy.wast:983
assert_return(() => call($10, "load8_u", [49_571]), 0);

// memory_copy.wast:984
assert_return(() => call($10, "load8_u", [49_770]), 0);

// memory_copy.wast:985
assert_return(() => call($10, "load8_u", [49_969]), 0);

// memory_copy.wast:986
assert_return(() => call($10, "load8_u", [50_168]), 0);

// memory_copy.wast:987
assert_return(() => call($10, "load8_u", [50_367]), 0);

// memory_copy.wast:988
assert_return(() => call($10, "load8_u", [50_566]), 0);

// memory_copy.wast:989
assert_return(() => call($10, "load8_u", [50_765]), 0);

// memory_copy.wast:990
assert_return(() => call($10, "load8_u", [50_964]), 0);

// memory_copy.wast:991
assert_return(() => call($10, "load8_u", [51_163]), 0);

// memory_copy.wast:992
assert_return(() => call($10, "load8_u", [51_362]), 0);

// memory_copy.wast:993
assert_return(() => call($10, "load8_u", [51_561]), 0);

// memory_copy.wast:994
assert_return(() => call($10, "load8_u", [51_760]), 0);

// memory_copy.wast:995
assert_return(() => call($10, "load8_u", [51_959]), 0);

// memory_copy.wast:996
assert_return(() => call($10, "load8_u", [52_158]), 0);

// memory_copy.wast:997
assert_return(() => call($10, "load8_u", [52_357]), 0);

// memory_copy.wast:998
assert_return(() => call($10, "load8_u", [52_556]), 0);

// memory_copy.wast:999
assert_return(() => call($10, "load8_u", [52_755]), 0);

// memory_copy.wast:1000
assert_return(() => call($10, "load8_u", [52_954]), 0);

// memory_copy.wast:1001
assert_return(() => call($10, "load8_u", [53_153]), 0);

// memory_copy.wast:1002
assert_return(() => call($10, "load8_u", [53_352]), 0);

// memory_copy.wast:1003
assert_return(() => call($10, "load8_u", [53_551]), 0);

// memory_copy.wast:1004
assert_return(() => call($10, "load8_u", [53_750]), 0);

// memory_copy.wast:1005
assert_return(() => call($10, "load8_u", [53_949]), 0);

// memory_copy.wast:1006
assert_return(() => call($10, "load8_u", [54_148]), 0);

// memory_copy.wast:1007
assert_return(() => call($10, "load8_u", [54_347]), 0);

// memory_copy.wast:1008
assert_return(() => call($10, "load8_u", [54_546]), 0);

// memory_copy.wast:1009
assert_return(() => call($10, "load8_u", [54_745]), 0);

// memory_copy.wast:1010
assert_return(() => call($10, "load8_u", [54_944]), 0);

// memory_copy.wast:1011
assert_return(() => call($10, "load8_u", [55_143]), 0);

// memory_copy.wast:1012
assert_return(() => call($10, "load8_u", [55_342]), 0);

// memory_copy.wast:1013
assert_return(() => call($10, "load8_u", [55_541]), 0);

// memory_copy.wast:1014
assert_return(() => call($10, "load8_u", [55_740]), 0);

// memory_copy.wast:1015
assert_return(() => call($10, "load8_u", [55_939]), 0);

// memory_copy.wast:1016
assert_return(() => call($10, "load8_u", [56_138]), 0);

// memory_copy.wast:1017
assert_return(() => call($10, "load8_u", [56_337]), 0);

// memory_copy.wast:1018
assert_return(() => call($10, "load8_u", [56_536]), 0);

// memory_copy.wast:1019
assert_return(() => call($10, "load8_u", [56_735]), 0);

// memory_copy.wast:1020
assert_return(() => call($10, "load8_u", [56_934]), 0);

// memory_copy.wast:1021
assert_return(() => call($10, "load8_u", [57_133]), 0);

// memory_copy.wast:1022
assert_return(() => call($10, "load8_u", [57_332]), 0);

// memory_copy.wast:1023
assert_return(() => call($10, "load8_u", [57_531]), 0);

// memory_copy.wast:1024
assert_return(() => call($10, "load8_u", [57_730]), 0);

// memory_copy.wast:1025
assert_return(() => call($10, "load8_u", [57_929]), 0);

// memory_copy.wast:1026
assert_return(() => call($10, "load8_u", [58_128]), 0);

// memory_copy.wast:1027
assert_return(() => call($10, "load8_u", [58_327]), 0);

// memory_copy.wast:1028
assert_return(() => call($10, "load8_u", [58_526]), 0);

// memory_copy.wast:1029
assert_return(() => call($10, "load8_u", [58_725]), 0);

// memory_copy.wast:1030
assert_return(() => call($10, "load8_u", [58_924]), 0);

// memory_copy.wast:1031
assert_return(() => call($10, "load8_u", [59_123]), 0);

// memory_copy.wast:1032
assert_return(() => call($10, "load8_u", [59_322]), 0);

// memory_copy.wast:1033
assert_return(() => call($10, "load8_u", [59_521]), 0);

// memory_copy.wast:1034
assert_return(() => call($10, "load8_u", [59_720]), 0);

// memory_copy.wast:1035
assert_return(() => call($10, "load8_u", [59_919]), 0);

// memory_copy.wast:1036
assert_return(() => call($10, "load8_u", [60_118]), 0);

// memory_copy.wast:1037
assert_return(() => call($10, "load8_u", [60_317]), 0);

// memory_copy.wast:1038
assert_return(() => call($10, "load8_u", [60_516]), 0);

// memory_copy.wast:1039
assert_return(() => call($10, "load8_u", [60_715]), 0);

// memory_copy.wast:1040
assert_return(() => call($10, "load8_u", [60_914]), 0);

// memory_copy.wast:1041
assert_return(() => call($10, "load8_u", [61_113]), 0);

// memory_copy.wast:1042
assert_return(() => call($10, "load8_u", [61_312]), 0);

// memory_copy.wast:1043
assert_return(() => call($10, "load8_u", [61_511]), 0);

// memory_copy.wast:1044
assert_return(() => call($10, "load8_u", [61_710]), 0);

// memory_copy.wast:1045
assert_return(() => call($10, "load8_u", [61_909]), 0);

// memory_copy.wast:1046
assert_return(() => call($10, "load8_u", [62_108]), 0);

// memory_copy.wast:1047
assert_return(() => call($10, "load8_u", [62_307]), 0);

// memory_copy.wast:1048
assert_return(() => call($10, "load8_u", [62_506]), 0);

// memory_copy.wast:1049
assert_return(() => call($10, "load8_u", [62_705]), 0);

// memory_copy.wast:1050
assert_return(() => call($10, "load8_u", [62_904]), 0);

// memory_copy.wast:1051
assert_return(() => call($10, "load8_u", [63_103]), 0);

// memory_copy.wast:1052
assert_return(() => call($10, "load8_u", [63_302]), 0);

// memory_copy.wast:1053
assert_return(() => call($10, "load8_u", [63_501]), 0);

// memory_copy.wast:1054
assert_return(() => call($10, "load8_u", [63_700]), 0);

// memory_copy.wast:1055
assert_return(() => call($10, "load8_u", [63_899]), 0);

// memory_copy.wast:1056
assert_return(() => call($10, "load8_u", [64_098]), 0);

// memory_copy.wast:1057
assert_return(() => call($10, "load8_u", [64_297]), 0);

// memory_copy.wast:1058
assert_return(() => call($10, "load8_u", [64_496]), 0);

// memory_copy.wast:1059
assert_return(() => call($10, "load8_u", [64_695]), 0);

// memory_copy.wast:1060
assert_return(() => call($10, "load8_u", [64_894]), 0);

// memory_copy.wast:1061
assert_return(() => call($10, "load8_u", [65_093]), 0);

// memory_copy.wast:1062
assert_return(() => call($10, "load8_u", [65_292]), 0);

// memory_copy.wast:1063
assert_return(() => call($10, "load8_u", [65_491]), 0);

// memory_copy.wast:1065
let $11 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9c\x80\x80\x80\x00\x01\x00\x41\xec\xff\x03\x0b\x14\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

// memory_copy.wast:1073
assert_trap(() => call($11, "run", [0, 65_516, 40]));

// memory_copy.wast:1076
assert_return(() => call($11, "load8_u", [198]), 0);

// memory_copy.wast:1077
assert_return(() => call($11, "load8_u", [397]), 0);

// memory_copy.wast:1078
assert_return(() => call($11, "load8_u", [596]), 0);

// memory_copy.wast:1079
assert_return(() => call($11, "load8_u", [795]), 0);

// memory_copy.wast:1080
assert_return(() => call($11, "load8_u", [994]), 0);

// memory_copy.wast:1081
assert_return(() => call($11, "load8_u", [1_193]), 0);

// memory_copy.wast:1082
assert_return(() => call($11, "load8_u", [1_392]), 0);

// memory_copy.wast:1083
assert_return(() => call($11, "load8_u", [1_591]), 0);

// memory_copy.wast:1084
assert_return(() => call($11, "load8_u", [1_790]), 0);

// memory_copy.wast:1085
assert_return(() => call($11, "load8_u", [1_989]), 0);

// memory_copy.wast:1086
assert_return(() => call($11, "load8_u", [2_188]), 0);

// memory_copy.wast:1087
assert_return(() => call($11, "load8_u", [2_387]), 0);

// memory_copy.wast:1088
assert_return(() => call($11, "load8_u", [2_586]), 0);

// memory_copy.wast:1089
assert_return(() => call($11, "load8_u", [2_785]), 0);

// memory_copy.wast:1090
assert_return(() => call($11, "load8_u", [2_984]), 0);

// memory_copy.wast:1091
assert_return(() => call($11, "load8_u", [3_183]), 0);

// memory_copy.wast:1092
assert_return(() => call($11, "load8_u", [3_382]), 0);

// memory_copy.wast:1093
assert_return(() => call($11, "load8_u", [3_581]), 0);

// memory_copy.wast:1094
assert_return(() => call($11, "load8_u", [3_780]), 0);

// memory_copy.wast:1095
assert_return(() => call($11, "load8_u", [3_979]), 0);

// memory_copy.wast:1096
assert_return(() => call($11, "load8_u", [4_178]), 0);

// memory_copy.wast:1097
assert_return(() => call($11, "load8_u", [4_377]), 0);

// memory_copy.wast:1098
assert_return(() => call($11, "load8_u", [4_576]), 0);

// memory_copy.wast:1099
assert_return(() => call($11, "load8_u", [4_775]), 0);

// memory_copy.wast:1100
assert_return(() => call($11, "load8_u", [4_974]), 0);

// memory_copy.wast:1101
assert_return(() => call($11, "load8_u", [5_173]), 0);

// memory_copy.wast:1102
assert_return(() => call($11, "load8_u", [5_372]), 0);

// memory_copy.wast:1103
assert_return(() => call($11, "load8_u", [5_571]), 0);

// memory_copy.wast:1104
assert_return(() => call($11, "load8_u", [5_770]), 0);

// memory_copy.wast:1105
assert_return(() => call($11, "load8_u", [5_969]), 0);

// memory_copy.wast:1106
assert_return(() => call($11, "load8_u", [6_168]), 0);

// memory_copy.wast:1107
assert_return(() => call($11, "load8_u", [6_367]), 0);

// memory_copy.wast:1108
assert_return(() => call($11, "load8_u", [6_566]), 0);

// memory_copy.wast:1109
assert_return(() => call($11, "load8_u", [6_765]), 0);

// memory_copy.wast:1110
assert_return(() => call($11, "load8_u", [6_964]), 0);

// memory_copy.wast:1111
assert_return(() => call($11, "load8_u", [7_163]), 0);

// memory_copy.wast:1112
assert_return(() => call($11, "load8_u", [7_362]), 0);

// memory_copy.wast:1113
assert_return(() => call($11, "load8_u", [7_561]), 0);

// memory_copy.wast:1114
assert_return(() => call($11, "load8_u", [7_760]), 0);

// memory_copy.wast:1115
assert_return(() => call($11, "load8_u", [7_959]), 0);

// memory_copy.wast:1116
assert_return(() => call($11, "load8_u", [8_158]), 0);

// memory_copy.wast:1117
assert_return(() => call($11, "load8_u", [8_357]), 0);

// memory_copy.wast:1118
assert_return(() => call($11, "load8_u", [8_556]), 0);

// memory_copy.wast:1119
assert_return(() => call($11, "load8_u", [8_755]), 0);

// memory_copy.wast:1120
assert_return(() => call($11, "load8_u", [8_954]), 0);

// memory_copy.wast:1121
assert_return(() => call($11, "load8_u", [9_153]), 0);

// memory_copy.wast:1122
assert_return(() => call($11, "load8_u", [9_352]), 0);

// memory_copy.wast:1123
assert_return(() => call($11, "load8_u", [9_551]), 0);

// memory_copy.wast:1124
assert_return(() => call($11, "load8_u", [9_750]), 0);

// memory_copy.wast:1125
assert_return(() => call($11, "load8_u", [9_949]), 0);

// memory_copy.wast:1126
assert_return(() => call($11, "load8_u", [10_148]), 0);

// memory_copy.wast:1127
assert_return(() => call($11, "load8_u", [10_347]), 0);

// memory_copy.wast:1128
assert_return(() => call($11, "load8_u", [10_546]), 0);

// memory_copy.wast:1129
assert_return(() => call($11, "load8_u", [10_745]), 0);

// memory_copy.wast:1130
assert_return(() => call($11, "load8_u", [10_944]), 0);

// memory_copy.wast:1131
assert_return(() => call($11, "load8_u", [11_143]), 0);

// memory_copy.wast:1132
assert_return(() => call($11, "load8_u", [11_342]), 0);

// memory_copy.wast:1133
assert_return(() => call($11, "load8_u", [11_541]), 0);

// memory_copy.wast:1134
assert_return(() => call($11, "load8_u", [11_740]), 0);

// memory_copy.wast:1135
assert_return(() => call($11, "load8_u", [11_939]), 0);

// memory_copy.wast:1136
assert_return(() => call($11, "load8_u", [12_138]), 0);

// memory_copy.wast:1137
assert_return(() => call($11, "load8_u", [12_337]), 0);

// memory_copy.wast:1138
assert_return(() => call($11, "load8_u", [12_536]), 0);

// memory_copy.wast:1139
assert_return(() => call($11, "load8_u", [12_735]), 0);

// memory_copy.wast:1140
assert_return(() => call($11, "load8_u", [12_934]), 0);

// memory_copy.wast:1141
assert_return(() => call($11, "load8_u", [13_133]), 0);

// memory_copy.wast:1142
assert_return(() => call($11, "load8_u", [13_332]), 0);

// memory_copy.wast:1143
assert_return(() => call($11, "load8_u", [13_531]), 0);

// memory_copy.wast:1144
assert_return(() => call($11, "load8_u", [13_730]), 0);

// memory_copy.wast:1145
assert_return(() => call($11, "load8_u", [13_929]), 0);

// memory_copy.wast:1146
assert_return(() => call($11, "load8_u", [14_128]), 0);

// memory_copy.wast:1147
assert_return(() => call($11, "load8_u", [14_327]), 0);

// memory_copy.wast:1148
assert_return(() => call($11, "load8_u", [14_526]), 0);

// memory_copy.wast:1149
assert_return(() => call($11, "load8_u", [14_725]), 0);

// memory_copy.wast:1150
assert_return(() => call($11, "load8_u", [14_924]), 0);

// memory_copy.wast:1151
assert_return(() => call($11, "load8_u", [15_123]), 0);

// memory_copy.wast:1152
assert_return(() => call($11, "load8_u", [15_322]), 0);

// memory_copy.wast:1153
assert_return(() => call($11, "load8_u", [15_521]), 0);

// memory_copy.wast:1154
assert_return(() => call($11, "load8_u", [15_720]), 0);

// memory_copy.wast:1155
assert_return(() => call($11, "load8_u", [15_919]), 0);

// memory_copy.wast:1156
assert_return(() => call($11, "load8_u", [16_118]), 0);

// memory_copy.wast:1157
assert_return(() => call($11, "load8_u", [16_317]), 0);

// memory_copy.wast:1158
assert_return(() => call($11, "load8_u", [16_516]), 0);

// memory_copy.wast:1159
assert_return(() => call($11, "load8_u", [16_715]), 0);

// memory_copy.wast:1160
assert_return(() => call($11, "load8_u", [16_914]), 0);

// memory_copy.wast:1161
assert_return(() => call($11, "load8_u", [17_113]), 0);

// memory_copy.wast:1162
assert_return(() => call($11, "load8_u", [17_312]), 0);

// memory_copy.wast:1163
assert_return(() => call($11, "load8_u", [17_511]), 0);

// memory_copy.wast:1164
assert_return(() => call($11, "load8_u", [17_710]), 0);

// memory_copy.wast:1165
assert_return(() => call($11, "load8_u", [17_909]), 0);

// memory_copy.wast:1166
assert_return(() => call($11, "load8_u", [18_108]), 0);

// memory_copy.wast:1167
assert_return(() => call($11, "load8_u", [18_307]), 0);

// memory_copy.wast:1168
assert_return(() => call($11, "load8_u", [18_506]), 0);

// memory_copy.wast:1169
assert_return(() => call($11, "load8_u", [18_705]), 0);

// memory_copy.wast:1170
assert_return(() => call($11, "load8_u", [18_904]), 0);

// memory_copy.wast:1171
assert_return(() => call($11, "load8_u", [19_103]), 0);

// memory_copy.wast:1172
assert_return(() => call($11, "load8_u", [19_302]), 0);

// memory_copy.wast:1173
assert_return(() => call($11, "load8_u", [19_501]), 0);

// memory_copy.wast:1174
assert_return(() => call($11, "load8_u", [19_700]), 0);

// memory_copy.wast:1175
assert_return(() => call($11, "load8_u", [19_899]), 0);

// memory_copy.wast:1176
assert_return(() => call($11, "load8_u", [20_098]), 0);

// memory_copy.wast:1177
assert_return(() => call($11, "load8_u", [20_297]), 0);

// memory_copy.wast:1178
assert_return(() => call($11, "load8_u", [20_496]), 0);

// memory_copy.wast:1179
assert_return(() => call($11, "load8_u", [20_695]), 0);

// memory_copy.wast:1180
assert_return(() => call($11, "load8_u", [20_894]), 0);

// memory_copy.wast:1181
assert_return(() => call($11, "load8_u", [21_093]), 0);

// memory_copy.wast:1182
assert_return(() => call($11, "load8_u", [21_292]), 0);

// memory_copy.wast:1183
assert_return(() => call($11, "load8_u", [21_491]), 0);

// memory_copy.wast:1184
assert_return(() => call($11, "load8_u", [21_690]), 0);

// memory_copy.wast:1185
assert_return(() => call($11, "load8_u", [21_889]), 0);

// memory_copy.wast:1186
assert_return(() => call($11, "load8_u", [22_088]), 0);

// memory_copy.wast:1187
assert_return(() => call($11, "load8_u", [22_287]), 0);

// memory_copy.wast:1188
assert_return(() => call($11, "load8_u", [22_486]), 0);

// memory_copy.wast:1189
assert_return(() => call($11, "load8_u", [22_685]), 0);

// memory_copy.wast:1190
assert_return(() => call($11, "load8_u", [22_884]), 0);

// memory_copy.wast:1191
assert_return(() => call($11, "load8_u", [23_083]), 0);

// memory_copy.wast:1192
assert_return(() => call($11, "load8_u", [23_282]), 0);

// memory_copy.wast:1193
assert_return(() => call($11, "load8_u", [23_481]), 0);

// memory_copy.wast:1194
assert_return(() => call($11, "load8_u", [23_680]), 0);

// memory_copy.wast:1195
assert_return(() => call($11, "load8_u", [23_879]), 0);

// memory_copy.wast:1196
assert_return(() => call($11, "load8_u", [24_078]), 0);

// memory_copy.wast:1197
assert_return(() => call($11, "load8_u", [24_277]), 0);

// memory_copy.wast:1198
assert_return(() => call($11, "load8_u", [24_476]), 0);

// memory_copy.wast:1199
assert_return(() => call($11, "load8_u", [24_675]), 0);

// memory_copy.wast:1200
assert_return(() => call($11, "load8_u", [24_874]), 0);

// memory_copy.wast:1201
assert_return(() => call($11, "load8_u", [25_073]), 0);

// memory_copy.wast:1202
assert_return(() => call($11, "load8_u", [25_272]), 0);

// memory_copy.wast:1203
assert_return(() => call($11, "load8_u", [25_471]), 0);

// memory_copy.wast:1204
assert_return(() => call($11, "load8_u", [25_670]), 0);

// memory_copy.wast:1205
assert_return(() => call($11, "load8_u", [25_869]), 0);

// memory_copy.wast:1206
assert_return(() => call($11, "load8_u", [26_068]), 0);

// memory_copy.wast:1207
assert_return(() => call($11, "load8_u", [26_267]), 0);

// memory_copy.wast:1208
assert_return(() => call($11, "load8_u", [26_466]), 0);

// memory_copy.wast:1209
assert_return(() => call($11, "load8_u", [26_665]), 0);

// memory_copy.wast:1210
assert_return(() => call($11, "load8_u", [26_864]), 0);

// memory_copy.wast:1211
assert_return(() => call($11, "load8_u", [27_063]), 0);

// memory_copy.wast:1212
assert_return(() => call($11, "load8_u", [27_262]), 0);

// memory_copy.wast:1213
assert_return(() => call($11, "load8_u", [27_461]), 0);

// memory_copy.wast:1214
assert_return(() => call($11, "load8_u", [27_660]), 0);

// memory_copy.wast:1215
assert_return(() => call($11, "load8_u", [27_859]), 0);

// memory_copy.wast:1216
assert_return(() => call($11, "load8_u", [28_058]), 0);

// memory_copy.wast:1217
assert_return(() => call($11, "load8_u", [28_257]), 0);

// memory_copy.wast:1218
assert_return(() => call($11, "load8_u", [28_456]), 0);

// memory_copy.wast:1219
assert_return(() => call($11, "load8_u", [28_655]), 0);

// memory_copy.wast:1220
assert_return(() => call($11, "load8_u", [28_854]), 0);

// memory_copy.wast:1221
assert_return(() => call($11, "load8_u", [29_053]), 0);

// memory_copy.wast:1222
assert_return(() => call($11, "load8_u", [29_252]), 0);

// memory_copy.wast:1223
assert_return(() => call($11, "load8_u", [29_451]), 0);

// memory_copy.wast:1224
assert_return(() => call($11, "load8_u", [29_650]), 0);

// memory_copy.wast:1225
assert_return(() => call($11, "load8_u", [29_849]), 0);

// memory_copy.wast:1226
assert_return(() => call($11, "load8_u", [30_048]), 0);

// memory_copy.wast:1227
assert_return(() => call($11, "load8_u", [30_247]), 0);

// memory_copy.wast:1228
assert_return(() => call($11, "load8_u", [30_446]), 0);

// memory_copy.wast:1229
assert_return(() => call($11, "load8_u", [30_645]), 0);

// memory_copy.wast:1230
assert_return(() => call($11, "load8_u", [30_844]), 0);

// memory_copy.wast:1231
assert_return(() => call($11, "load8_u", [31_043]), 0);

// memory_copy.wast:1232
assert_return(() => call($11, "load8_u", [31_242]), 0);

// memory_copy.wast:1233
assert_return(() => call($11, "load8_u", [31_441]), 0);

// memory_copy.wast:1234
assert_return(() => call($11, "load8_u", [31_640]), 0);

// memory_copy.wast:1235
assert_return(() => call($11, "load8_u", [31_839]), 0);

// memory_copy.wast:1236
assert_return(() => call($11, "load8_u", [32_038]), 0);

// memory_copy.wast:1237
assert_return(() => call($11, "load8_u", [32_237]), 0);

// memory_copy.wast:1238
assert_return(() => call($11, "load8_u", [32_436]), 0);

// memory_copy.wast:1239
assert_return(() => call($11, "load8_u", [32_635]), 0);

// memory_copy.wast:1240
assert_return(() => call($11, "load8_u", [32_834]), 0);

// memory_copy.wast:1241
assert_return(() => call($11, "load8_u", [33_033]), 0);

// memory_copy.wast:1242
assert_return(() => call($11, "load8_u", [33_232]), 0);

// memory_copy.wast:1243
assert_return(() => call($11, "load8_u", [33_431]), 0);

// memory_copy.wast:1244
assert_return(() => call($11, "load8_u", [33_630]), 0);

// memory_copy.wast:1245
assert_return(() => call($11, "load8_u", [33_829]), 0);

// memory_copy.wast:1246
assert_return(() => call($11, "load8_u", [34_028]), 0);

// memory_copy.wast:1247
assert_return(() => call($11, "load8_u", [34_227]), 0);

// memory_copy.wast:1248
assert_return(() => call($11, "load8_u", [34_426]), 0);

// memory_copy.wast:1249
assert_return(() => call($11, "load8_u", [34_625]), 0);

// memory_copy.wast:1250
assert_return(() => call($11, "load8_u", [34_824]), 0);

// memory_copy.wast:1251
assert_return(() => call($11, "load8_u", [35_023]), 0);

// memory_copy.wast:1252
assert_return(() => call($11, "load8_u", [35_222]), 0);

// memory_copy.wast:1253
assert_return(() => call($11, "load8_u", [35_421]), 0);

// memory_copy.wast:1254
assert_return(() => call($11, "load8_u", [35_620]), 0);

// memory_copy.wast:1255
assert_return(() => call($11, "load8_u", [35_819]), 0);

// memory_copy.wast:1256
assert_return(() => call($11, "load8_u", [36_018]), 0);

// memory_copy.wast:1257
assert_return(() => call($11, "load8_u", [36_217]), 0);

// memory_copy.wast:1258
assert_return(() => call($11, "load8_u", [36_416]), 0);

// memory_copy.wast:1259
assert_return(() => call($11, "load8_u", [36_615]), 0);

// memory_copy.wast:1260
assert_return(() => call($11, "load8_u", [36_814]), 0);

// memory_copy.wast:1261
assert_return(() => call($11, "load8_u", [37_013]), 0);

// memory_copy.wast:1262
assert_return(() => call($11, "load8_u", [37_212]), 0);

// memory_copy.wast:1263
assert_return(() => call($11, "load8_u", [37_411]), 0);

// memory_copy.wast:1264
assert_return(() => call($11, "load8_u", [37_610]), 0);

// memory_copy.wast:1265
assert_return(() => call($11, "load8_u", [37_809]), 0);

// memory_copy.wast:1266
assert_return(() => call($11, "load8_u", [38_008]), 0);

// memory_copy.wast:1267
assert_return(() => call($11, "load8_u", [38_207]), 0);

// memory_copy.wast:1268
assert_return(() => call($11, "load8_u", [38_406]), 0);

// memory_copy.wast:1269
assert_return(() => call($11, "load8_u", [38_605]), 0);

// memory_copy.wast:1270
assert_return(() => call($11, "load8_u", [38_804]), 0);

// memory_copy.wast:1271
assert_return(() => call($11, "load8_u", [39_003]), 0);

// memory_copy.wast:1272
assert_return(() => call($11, "load8_u", [39_202]), 0);

// memory_copy.wast:1273
assert_return(() => call($11, "load8_u", [39_401]), 0);

// memory_copy.wast:1274
assert_return(() => call($11, "load8_u", [39_600]), 0);

// memory_copy.wast:1275
assert_return(() => call($11, "load8_u", [39_799]), 0);

// memory_copy.wast:1276
assert_return(() => call($11, "load8_u", [39_998]), 0);

// memory_copy.wast:1277
assert_return(() => call($11, "load8_u", [40_197]), 0);

// memory_copy.wast:1278
assert_return(() => call($11, "load8_u", [40_396]), 0);

// memory_copy.wast:1279
assert_return(() => call($11, "load8_u", [40_595]), 0);

// memory_copy.wast:1280
assert_return(() => call($11, "load8_u", [40_794]), 0);

// memory_copy.wast:1281
assert_return(() => call($11, "load8_u", [40_993]), 0);

// memory_copy.wast:1282
assert_return(() => call($11, "load8_u", [41_192]), 0);

// memory_copy.wast:1283
assert_return(() => call($11, "load8_u", [41_391]), 0);

// memory_copy.wast:1284
assert_return(() => call($11, "load8_u", [41_590]), 0);

// memory_copy.wast:1285
assert_return(() => call($11, "load8_u", [41_789]), 0);

// memory_copy.wast:1286
assert_return(() => call($11, "load8_u", [41_988]), 0);

// memory_copy.wast:1287
assert_return(() => call($11, "load8_u", [42_187]), 0);

// memory_copy.wast:1288
assert_return(() => call($11, "load8_u", [42_386]), 0);

// memory_copy.wast:1289
assert_return(() => call($11, "load8_u", [42_585]), 0);

// memory_copy.wast:1290
assert_return(() => call($11, "load8_u", [42_784]), 0);

// memory_copy.wast:1291
assert_return(() => call($11, "load8_u", [42_983]), 0);

// memory_copy.wast:1292
assert_return(() => call($11, "load8_u", [43_182]), 0);

// memory_copy.wast:1293
assert_return(() => call($11, "load8_u", [43_381]), 0);

// memory_copy.wast:1294
assert_return(() => call($11, "load8_u", [43_580]), 0);

// memory_copy.wast:1295
assert_return(() => call($11, "load8_u", [43_779]), 0);

// memory_copy.wast:1296
assert_return(() => call($11, "load8_u", [43_978]), 0);

// memory_copy.wast:1297
assert_return(() => call($11, "load8_u", [44_177]), 0);

// memory_copy.wast:1298
assert_return(() => call($11, "load8_u", [44_376]), 0);

// memory_copy.wast:1299
assert_return(() => call($11, "load8_u", [44_575]), 0);

// memory_copy.wast:1300
assert_return(() => call($11, "load8_u", [44_774]), 0);

// memory_copy.wast:1301
assert_return(() => call($11, "load8_u", [44_973]), 0);

// memory_copy.wast:1302
assert_return(() => call($11, "load8_u", [45_172]), 0);

// memory_copy.wast:1303
assert_return(() => call($11, "load8_u", [45_371]), 0);

// memory_copy.wast:1304
assert_return(() => call($11, "load8_u", [45_570]), 0);

// memory_copy.wast:1305
assert_return(() => call($11, "load8_u", [45_769]), 0);

// memory_copy.wast:1306
assert_return(() => call($11, "load8_u", [45_968]), 0);

// memory_copy.wast:1307
assert_return(() => call($11, "load8_u", [46_167]), 0);

// memory_copy.wast:1308
assert_return(() => call($11, "load8_u", [46_366]), 0);

// memory_copy.wast:1309
assert_return(() => call($11, "load8_u", [46_565]), 0);

// memory_copy.wast:1310
assert_return(() => call($11, "load8_u", [46_764]), 0);

// memory_copy.wast:1311
assert_return(() => call($11, "load8_u", [46_963]), 0);

// memory_copy.wast:1312
assert_return(() => call($11, "load8_u", [47_162]), 0);

// memory_copy.wast:1313
assert_return(() => call($11, "load8_u", [47_361]), 0);

// memory_copy.wast:1314
assert_return(() => call($11, "load8_u", [47_560]), 0);

// memory_copy.wast:1315
assert_return(() => call($11, "load8_u", [47_759]), 0);

// memory_copy.wast:1316
assert_return(() => call($11, "load8_u", [47_958]), 0);

// memory_copy.wast:1317
assert_return(() => call($11, "load8_u", [48_157]), 0);

// memory_copy.wast:1318
assert_return(() => call($11, "load8_u", [48_356]), 0);

// memory_copy.wast:1319
assert_return(() => call($11, "load8_u", [48_555]), 0);

// memory_copy.wast:1320
assert_return(() => call($11, "load8_u", [48_754]), 0);

// memory_copy.wast:1321
assert_return(() => call($11, "load8_u", [48_953]), 0);

// memory_copy.wast:1322
assert_return(() => call($11, "load8_u", [49_152]), 0);

// memory_copy.wast:1323
assert_return(() => call($11, "load8_u", [49_351]), 0);

// memory_copy.wast:1324
assert_return(() => call($11, "load8_u", [49_550]), 0);

// memory_copy.wast:1325
assert_return(() => call($11, "load8_u", [49_749]), 0);

// memory_copy.wast:1326
assert_return(() => call($11, "load8_u", [49_948]), 0);

// memory_copy.wast:1327
assert_return(() => call($11, "load8_u", [50_147]), 0);

// memory_copy.wast:1328
assert_return(() => call($11, "load8_u", [50_346]), 0);

// memory_copy.wast:1329
assert_return(() => call($11, "load8_u", [50_545]), 0);

// memory_copy.wast:1330
assert_return(() => call($11, "load8_u", [50_744]), 0);

// memory_copy.wast:1331
assert_return(() => call($11, "load8_u", [50_943]), 0);

// memory_copy.wast:1332
assert_return(() => call($11, "load8_u", [51_142]), 0);

// memory_copy.wast:1333
assert_return(() => call($11, "load8_u", [51_341]), 0);

// memory_copy.wast:1334
assert_return(() => call($11, "load8_u", [51_540]), 0);

// memory_copy.wast:1335
assert_return(() => call($11, "load8_u", [51_739]), 0);

// memory_copy.wast:1336
assert_return(() => call($11, "load8_u", [51_938]), 0);

// memory_copy.wast:1337
assert_return(() => call($11, "load8_u", [52_137]), 0);

// memory_copy.wast:1338
assert_return(() => call($11, "load8_u", [52_336]), 0);

// memory_copy.wast:1339
assert_return(() => call($11, "load8_u", [52_535]), 0);

// memory_copy.wast:1340
assert_return(() => call($11, "load8_u", [52_734]), 0);

// memory_copy.wast:1341
assert_return(() => call($11, "load8_u", [52_933]), 0);

// memory_copy.wast:1342
assert_return(() => call($11, "load8_u", [53_132]), 0);

// memory_copy.wast:1343
assert_return(() => call($11, "load8_u", [53_331]), 0);

// memory_copy.wast:1344
assert_return(() => call($11, "load8_u", [53_530]), 0);

// memory_copy.wast:1345
assert_return(() => call($11, "load8_u", [53_729]), 0);

// memory_copy.wast:1346
assert_return(() => call($11, "load8_u", [53_928]), 0);

// memory_copy.wast:1347
assert_return(() => call($11, "load8_u", [54_127]), 0);

// memory_copy.wast:1348
assert_return(() => call($11, "load8_u", [54_326]), 0);

// memory_copy.wast:1349
assert_return(() => call($11, "load8_u", [54_525]), 0);

// memory_copy.wast:1350
assert_return(() => call($11, "load8_u", [54_724]), 0);

// memory_copy.wast:1351
assert_return(() => call($11, "load8_u", [54_923]), 0);

// memory_copy.wast:1352
assert_return(() => call($11, "load8_u", [55_122]), 0);

// memory_copy.wast:1353
assert_return(() => call($11, "load8_u", [55_321]), 0);

// memory_copy.wast:1354
assert_return(() => call($11, "load8_u", [55_520]), 0);

// memory_copy.wast:1355
assert_return(() => call($11, "load8_u", [55_719]), 0);

// memory_copy.wast:1356
assert_return(() => call($11, "load8_u", [55_918]), 0);

// memory_copy.wast:1357
assert_return(() => call($11, "load8_u", [56_117]), 0);

// memory_copy.wast:1358
assert_return(() => call($11, "load8_u", [56_316]), 0);

// memory_copy.wast:1359
assert_return(() => call($11, "load8_u", [56_515]), 0);

// memory_copy.wast:1360
assert_return(() => call($11, "load8_u", [56_714]), 0);

// memory_copy.wast:1361
assert_return(() => call($11, "load8_u", [56_913]), 0);

// memory_copy.wast:1362
assert_return(() => call($11, "load8_u", [57_112]), 0);

// memory_copy.wast:1363
assert_return(() => call($11, "load8_u", [57_311]), 0);

// memory_copy.wast:1364
assert_return(() => call($11, "load8_u", [57_510]), 0);

// memory_copy.wast:1365
assert_return(() => call($11, "load8_u", [57_709]), 0);

// memory_copy.wast:1366
assert_return(() => call($11, "load8_u", [57_908]), 0);

// memory_copy.wast:1367
assert_return(() => call($11, "load8_u", [58_107]), 0);

// memory_copy.wast:1368
assert_return(() => call($11, "load8_u", [58_306]), 0);

// memory_copy.wast:1369
assert_return(() => call($11, "load8_u", [58_505]), 0);

// memory_copy.wast:1370
assert_return(() => call($11, "load8_u", [58_704]), 0);

// memory_copy.wast:1371
assert_return(() => call($11, "load8_u", [58_903]), 0);

// memory_copy.wast:1372
assert_return(() => call($11, "load8_u", [59_102]), 0);

// memory_copy.wast:1373
assert_return(() => call($11, "load8_u", [59_301]), 0);

// memory_copy.wast:1374
assert_return(() => call($11, "load8_u", [59_500]), 0);

// memory_copy.wast:1375
assert_return(() => call($11, "load8_u", [59_699]), 0);

// memory_copy.wast:1376
assert_return(() => call($11, "load8_u", [59_898]), 0);

// memory_copy.wast:1377
assert_return(() => call($11, "load8_u", [60_097]), 0);

// memory_copy.wast:1378
assert_return(() => call($11, "load8_u", [60_296]), 0);

// memory_copy.wast:1379
assert_return(() => call($11, "load8_u", [60_495]), 0);

// memory_copy.wast:1380
assert_return(() => call($11, "load8_u", [60_694]), 0);

// memory_copy.wast:1381
assert_return(() => call($11, "load8_u", [60_893]), 0);

// memory_copy.wast:1382
assert_return(() => call($11, "load8_u", [61_092]), 0);

// memory_copy.wast:1383
assert_return(() => call($11, "load8_u", [61_291]), 0);

// memory_copy.wast:1384
assert_return(() => call($11, "load8_u", [61_490]), 0);

// memory_copy.wast:1385
assert_return(() => call($11, "load8_u", [61_689]), 0);

// memory_copy.wast:1386
assert_return(() => call($11, "load8_u", [61_888]), 0);

// memory_copy.wast:1387
assert_return(() => call($11, "load8_u", [62_087]), 0);

// memory_copy.wast:1388
assert_return(() => call($11, "load8_u", [62_286]), 0);

// memory_copy.wast:1389
assert_return(() => call($11, "load8_u", [62_485]), 0);

// memory_copy.wast:1390
assert_return(() => call($11, "load8_u", [62_684]), 0);

// memory_copy.wast:1391
assert_return(() => call($11, "load8_u", [62_883]), 0);

// memory_copy.wast:1392
assert_return(() => call($11, "load8_u", [63_082]), 0);

// memory_copy.wast:1393
assert_return(() => call($11, "load8_u", [63_281]), 0);

// memory_copy.wast:1394
assert_return(() => call($11, "load8_u", [63_480]), 0);

// memory_copy.wast:1395
assert_return(() => call($11, "load8_u", [63_679]), 0);

// memory_copy.wast:1396
assert_return(() => call($11, "load8_u", [63_878]), 0);

// memory_copy.wast:1397
assert_return(() => call($11, "load8_u", [64_077]), 0);

// memory_copy.wast:1398
assert_return(() => call($11, "load8_u", [64_276]), 0);

// memory_copy.wast:1399
assert_return(() => call($11, "load8_u", [64_475]), 0);

// memory_copy.wast:1400
assert_return(() => call($11, "load8_u", [64_674]), 0);

// memory_copy.wast:1401
assert_return(() => call($11, "load8_u", [64_873]), 0);

// memory_copy.wast:1402
assert_return(() => call($11, "load8_u", [65_072]), 0);

// memory_copy.wast:1403
assert_return(() => call($11, "load8_u", [65_271]), 0);

// memory_copy.wast:1404
assert_return(() => call($11, "load8_u", [65_470]), 0);

// memory_copy.wast:1405
assert_return(() => call($11, "load8_u", [65_516]), 0);

// memory_copy.wast:1406
assert_return(() => call($11, "load8_u", [65_517]), 1);

// memory_copy.wast:1407
assert_return(() => call($11, "load8_u", [65_518]), 2);

// memory_copy.wast:1408
assert_return(() => call($11, "load8_u", [65_519]), 3);

// memory_copy.wast:1409
assert_return(() => call($11, "load8_u", [65_520]), 4);

// memory_copy.wast:1410
assert_return(() => call($11, "load8_u", [65_521]), 5);

// memory_copy.wast:1411
assert_return(() => call($11, "load8_u", [65_522]), 6);

// memory_copy.wast:1412
assert_return(() => call($11, "load8_u", [65_523]), 7);

// memory_copy.wast:1413
assert_return(() => call($11, "load8_u", [65_524]), 8);

// memory_copy.wast:1414
assert_return(() => call($11, "load8_u", [65_525]), 9);

// memory_copy.wast:1415
assert_return(() => call($11, "load8_u", [65_526]), 10);

// memory_copy.wast:1416
assert_return(() => call($11, "load8_u", [65_527]), 11);

// memory_copy.wast:1417
assert_return(() => call($11, "load8_u", [65_528]), 12);

// memory_copy.wast:1418
assert_return(() => call($11, "load8_u", [65_529]), 13);

// memory_copy.wast:1419
assert_return(() => call($11, "load8_u", [65_530]), 14);

// memory_copy.wast:1420
assert_return(() => call($11, "load8_u", [65_531]), 15);

// memory_copy.wast:1421
assert_return(() => call($11, "load8_u", [65_532]), 16);

// memory_copy.wast:1422
assert_return(() => call($11, "load8_u", [65_533]), 17);

// memory_copy.wast:1423
assert_return(() => call($11, "load8_u", [65_534]), 18);

// memory_copy.wast:1424
assert_return(() => call($11, "load8_u", [65_535]), 19);

// memory_copy.wast:1426
let $12 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9d\x80\x80\x80\x00\x01\x00\x41\xeb\xff\x03\x0b\x15\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\x14");

// memory_copy.wast:1434
assert_trap(() => call($12, "run", [0, 65_515, 39]));

// memory_copy.wast:1437
assert_return(() => call($12, "load8_u", [198]), 0);

// memory_copy.wast:1438
assert_return(() => call($12, "load8_u", [397]), 0);

// memory_copy.wast:1439
assert_return(() => call($12, "load8_u", [596]), 0);

// memory_copy.wast:1440
assert_return(() => call($12, "load8_u", [795]), 0);

// memory_copy.wast:1441
assert_return(() => call($12, "load8_u", [994]), 0);

// memory_copy.wast:1442
assert_return(() => call($12, "load8_u", [1_193]), 0);

// memory_copy.wast:1443
assert_return(() => call($12, "load8_u", [1_392]), 0);

// memory_copy.wast:1444
assert_return(() => call($12, "load8_u", [1_591]), 0);

// memory_copy.wast:1445
assert_return(() => call($12, "load8_u", [1_790]), 0);

// memory_copy.wast:1446
assert_return(() => call($12, "load8_u", [1_989]), 0);

// memory_copy.wast:1447
assert_return(() => call($12, "load8_u", [2_188]), 0);

// memory_copy.wast:1448
assert_return(() => call($12, "load8_u", [2_387]), 0);

// memory_copy.wast:1449
assert_return(() => call($12, "load8_u", [2_586]), 0);

// memory_copy.wast:1450
assert_return(() => call($12, "load8_u", [2_785]), 0);

// memory_copy.wast:1451
assert_return(() => call($12, "load8_u", [2_984]), 0);

// memory_copy.wast:1452
assert_return(() => call($12, "load8_u", [3_183]), 0);

// memory_copy.wast:1453
assert_return(() => call($12, "load8_u", [3_382]), 0);

// memory_copy.wast:1454
assert_return(() => call($12, "load8_u", [3_581]), 0);

// memory_copy.wast:1455
assert_return(() => call($12, "load8_u", [3_780]), 0);

// memory_copy.wast:1456
assert_return(() => call($12, "load8_u", [3_979]), 0);

// memory_copy.wast:1457
assert_return(() => call($12, "load8_u", [4_178]), 0);

// memory_copy.wast:1458
assert_return(() => call($12, "load8_u", [4_377]), 0);

// memory_copy.wast:1459
assert_return(() => call($12, "load8_u", [4_576]), 0);

// memory_copy.wast:1460
assert_return(() => call($12, "load8_u", [4_775]), 0);

// memory_copy.wast:1461
assert_return(() => call($12, "load8_u", [4_974]), 0);

// memory_copy.wast:1462
assert_return(() => call($12, "load8_u", [5_173]), 0);

// memory_copy.wast:1463
assert_return(() => call($12, "load8_u", [5_372]), 0);

// memory_copy.wast:1464
assert_return(() => call($12, "load8_u", [5_571]), 0);

// memory_copy.wast:1465
assert_return(() => call($12, "load8_u", [5_770]), 0);

// memory_copy.wast:1466
assert_return(() => call($12, "load8_u", [5_969]), 0);

// memory_copy.wast:1467
assert_return(() => call($12, "load8_u", [6_168]), 0);

// memory_copy.wast:1468
assert_return(() => call($12, "load8_u", [6_367]), 0);

// memory_copy.wast:1469
assert_return(() => call($12, "load8_u", [6_566]), 0);

// memory_copy.wast:1470
assert_return(() => call($12, "load8_u", [6_765]), 0);

// memory_copy.wast:1471
assert_return(() => call($12, "load8_u", [6_964]), 0);

// memory_copy.wast:1472
assert_return(() => call($12, "load8_u", [7_163]), 0);

// memory_copy.wast:1473
assert_return(() => call($12, "load8_u", [7_362]), 0);

// memory_copy.wast:1474
assert_return(() => call($12, "load8_u", [7_561]), 0);

// memory_copy.wast:1475
assert_return(() => call($12, "load8_u", [7_760]), 0);

// memory_copy.wast:1476
assert_return(() => call($12, "load8_u", [7_959]), 0);

// memory_copy.wast:1477
assert_return(() => call($12, "load8_u", [8_158]), 0);

// memory_copy.wast:1478
assert_return(() => call($12, "load8_u", [8_357]), 0);

// memory_copy.wast:1479
assert_return(() => call($12, "load8_u", [8_556]), 0);

// memory_copy.wast:1480
assert_return(() => call($12, "load8_u", [8_755]), 0);

// memory_copy.wast:1481
assert_return(() => call($12, "load8_u", [8_954]), 0);

// memory_copy.wast:1482
assert_return(() => call($12, "load8_u", [9_153]), 0);

// memory_copy.wast:1483
assert_return(() => call($12, "load8_u", [9_352]), 0);

// memory_copy.wast:1484
assert_return(() => call($12, "load8_u", [9_551]), 0);

// memory_copy.wast:1485
assert_return(() => call($12, "load8_u", [9_750]), 0);

// memory_copy.wast:1486
assert_return(() => call($12, "load8_u", [9_949]), 0);

// memory_copy.wast:1487
assert_return(() => call($12, "load8_u", [10_148]), 0);

// memory_copy.wast:1488
assert_return(() => call($12, "load8_u", [10_347]), 0);

// memory_copy.wast:1489
assert_return(() => call($12, "load8_u", [10_546]), 0);

// memory_copy.wast:1490
assert_return(() => call($12, "load8_u", [10_745]), 0);

// memory_copy.wast:1491
assert_return(() => call($12, "load8_u", [10_944]), 0);

// memory_copy.wast:1492
assert_return(() => call($12, "load8_u", [11_143]), 0);

// memory_copy.wast:1493
assert_return(() => call($12, "load8_u", [11_342]), 0);

// memory_copy.wast:1494
assert_return(() => call($12, "load8_u", [11_541]), 0);

// memory_copy.wast:1495
assert_return(() => call($12, "load8_u", [11_740]), 0);

// memory_copy.wast:1496
assert_return(() => call($12, "load8_u", [11_939]), 0);

// memory_copy.wast:1497
assert_return(() => call($12, "load8_u", [12_138]), 0);

// memory_copy.wast:1498
assert_return(() => call($12, "load8_u", [12_337]), 0);

// memory_copy.wast:1499
assert_return(() => call($12, "load8_u", [12_536]), 0);

// memory_copy.wast:1500
assert_return(() => call($12, "load8_u", [12_735]), 0);

// memory_copy.wast:1501
assert_return(() => call($12, "load8_u", [12_934]), 0);

// memory_copy.wast:1502
assert_return(() => call($12, "load8_u", [13_133]), 0);

// memory_copy.wast:1503
assert_return(() => call($12, "load8_u", [13_332]), 0);

// memory_copy.wast:1504
assert_return(() => call($12, "load8_u", [13_531]), 0);

// memory_copy.wast:1505
assert_return(() => call($12, "load8_u", [13_730]), 0);

// memory_copy.wast:1506
assert_return(() => call($12, "load8_u", [13_929]), 0);

// memory_copy.wast:1507
assert_return(() => call($12, "load8_u", [14_128]), 0);

// memory_copy.wast:1508
assert_return(() => call($12, "load8_u", [14_327]), 0);

// memory_copy.wast:1509
assert_return(() => call($12, "load8_u", [14_526]), 0);

// memory_copy.wast:1510
assert_return(() => call($12, "load8_u", [14_725]), 0);

// memory_copy.wast:1511
assert_return(() => call($12, "load8_u", [14_924]), 0);

// memory_copy.wast:1512
assert_return(() => call($12, "load8_u", [15_123]), 0);

// memory_copy.wast:1513
assert_return(() => call($12, "load8_u", [15_322]), 0);

// memory_copy.wast:1514
assert_return(() => call($12, "load8_u", [15_521]), 0);

// memory_copy.wast:1515
assert_return(() => call($12, "load8_u", [15_720]), 0);

// memory_copy.wast:1516
assert_return(() => call($12, "load8_u", [15_919]), 0);

// memory_copy.wast:1517
assert_return(() => call($12, "load8_u", [16_118]), 0);

// memory_copy.wast:1518
assert_return(() => call($12, "load8_u", [16_317]), 0);

// memory_copy.wast:1519
assert_return(() => call($12, "load8_u", [16_516]), 0);

// memory_copy.wast:1520
assert_return(() => call($12, "load8_u", [16_715]), 0);

// memory_copy.wast:1521
assert_return(() => call($12, "load8_u", [16_914]), 0);

// memory_copy.wast:1522
assert_return(() => call($12, "load8_u", [17_113]), 0);

// memory_copy.wast:1523
assert_return(() => call($12, "load8_u", [17_312]), 0);

// memory_copy.wast:1524
assert_return(() => call($12, "load8_u", [17_511]), 0);

// memory_copy.wast:1525
assert_return(() => call($12, "load8_u", [17_710]), 0);

// memory_copy.wast:1526
assert_return(() => call($12, "load8_u", [17_909]), 0);

// memory_copy.wast:1527
assert_return(() => call($12, "load8_u", [18_108]), 0);

// memory_copy.wast:1528
assert_return(() => call($12, "load8_u", [18_307]), 0);

// memory_copy.wast:1529
assert_return(() => call($12, "load8_u", [18_506]), 0);

// memory_copy.wast:1530
assert_return(() => call($12, "load8_u", [18_705]), 0);

// memory_copy.wast:1531
assert_return(() => call($12, "load8_u", [18_904]), 0);

// memory_copy.wast:1532
assert_return(() => call($12, "load8_u", [19_103]), 0);

// memory_copy.wast:1533
assert_return(() => call($12, "load8_u", [19_302]), 0);

// memory_copy.wast:1534
assert_return(() => call($12, "load8_u", [19_501]), 0);

// memory_copy.wast:1535
assert_return(() => call($12, "load8_u", [19_700]), 0);

// memory_copy.wast:1536
assert_return(() => call($12, "load8_u", [19_899]), 0);

// memory_copy.wast:1537
assert_return(() => call($12, "load8_u", [20_098]), 0);

// memory_copy.wast:1538
assert_return(() => call($12, "load8_u", [20_297]), 0);

// memory_copy.wast:1539
assert_return(() => call($12, "load8_u", [20_496]), 0);

// memory_copy.wast:1540
assert_return(() => call($12, "load8_u", [20_695]), 0);

// memory_copy.wast:1541
assert_return(() => call($12, "load8_u", [20_894]), 0);

// memory_copy.wast:1542
assert_return(() => call($12, "load8_u", [21_093]), 0);

// memory_copy.wast:1543
assert_return(() => call($12, "load8_u", [21_292]), 0);

// memory_copy.wast:1544
assert_return(() => call($12, "load8_u", [21_491]), 0);

// memory_copy.wast:1545
assert_return(() => call($12, "load8_u", [21_690]), 0);

// memory_copy.wast:1546
assert_return(() => call($12, "load8_u", [21_889]), 0);

// memory_copy.wast:1547
assert_return(() => call($12, "load8_u", [22_088]), 0);

// memory_copy.wast:1548
assert_return(() => call($12, "load8_u", [22_287]), 0);

// memory_copy.wast:1549
assert_return(() => call($12, "load8_u", [22_486]), 0);

// memory_copy.wast:1550
assert_return(() => call($12, "load8_u", [22_685]), 0);

// memory_copy.wast:1551
assert_return(() => call($12, "load8_u", [22_884]), 0);

// memory_copy.wast:1552
assert_return(() => call($12, "load8_u", [23_083]), 0);

// memory_copy.wast:1553
assert_return(() => call($12, "load8_u", [23_282]), 0);

// memory_copy.wast:1554
assert_return(() => call($12, "load8_u", [23_481]), 0);

// memory_copy.wast:1555
assert_return(() => call($12, "load8_u", [23_680]), 0);

// memory_copy.wast:1556
assert_return(() => call($12, "load8_u", [23_879]), 0);

// memory_copy.wast:1557
assert_return(() => call($12, "load8_u", [24_078]), 0);

// memory_copy.wast:1558
assert_return(() => call($12, "load8_u", [24_277]), 0);

// memory_copy.wast:1559
assert_return(() => call($12, "load8_u", [24_476]), 0);

// memory_copy.wast:1560
assert_return(() => call($12, "load8_u", [24_675]), 0);

// memory_copy.wast:1561
assert_return(() => call($12, "load8_u", [24_874]), 0);

// memory_copy.wast:1562
assert_return(() => call($12, "load8_u", [25_073]), 0);

// memory_copy.wast:1563
assert_return(() => call($12, "load8_u", [25_272]), 0);

// memory_copy.wast:1564
assert_return(() => call($12, "load8_u", [25_471]), 0);

// memory_copy.wast:1565
assert_return(() => call($12, "load8_u", [25_670]), 0);

// memory_copy.wast:1566
assert_return(() => call($12, "load8_u", [25_869]), 0);

// memory_copy.wast:1567
assert_return(() => call($12, "load8_u", [26_068]), 0);

// memory_copy.wast:1568
assert_return(() => call($12, "load8_u", [26_267]), 0);

// memory_copy.wast:1569
assert_return(() => call($12, "load8_u", [26_466]), 0);

// memory_copy.wast:1570
assert_return(() => call($12, "load8_u", [26_665]), 0);

// memory_copy.wast:1571
assert_return(() => call($12, "load8_u", [26_864]), 0);

// memory_copy.wast:1572
assert_return(() => call($12, "load8_u", [27_063]), 0);

// memory_copy.wast:1573
assert_return(() => call($12, "load8_u", [27_262]), 0);

// memory_copy.wast:1574
assert_return(() => call($12, "load8_u", [27_461]), 0);

// memory_copy.wast:1575
assert_return(() => call($12, "load8_u", [27_660]), 0);

// memory_copy.wast:1576
assert_return(() => call($12, "load8_u", [27_859]), 0);

// memory_copy.wast:1577
assert_return(() => call($12, "load8_u", [28_058]), 0);

// memory_copy.wast:1578
assert_return(() => call($12, "load8_u", [28_257]), 0);

// memory_copy.wast:1579
assert_return(() => call($12, "load8_u", [28_456]), 0);

// memory_copy.wast:1580
assert_return(() => call($12, "load8_u", [28_655]), 0);

// memory_copy.wast:1581
assert_return(() => call($12, "load8_u", [28_854]), 0);

// memory_copy.wast:1582
assert_return(() => call($12, "load8_u", [29_053]), 0);

// memory_copy.wast:1583
assert_return(() => call($12, "load8_u", [29_252]), 0);

// memory_copy.wast:1584
assert_return(() => call($12, "load8_u", [29_451]), 0);

// memory_copy.wast:1585
assert_return(() => call($12, "load8_u", [29_650]), 0);

// memory_copy.wast:1586
assert_return(() => call($12, "load8_u", [29_849]), 0);

// memory_copy.wast:1587
assert_return(() => call($12, "load8_u", [30_048]), 0);

// memory_copy.wast:1588
assert_return(() => call($12, "load8_u", [30_247]), 0);

// memory_copy.wast:1589
assert_return(() => call($12, "load8_u", [30_446]), 0);

// memory_copy.wast:1590
assert_return(() => call($12, "load8_u", [30_645]), 0);

// memory_copy.wast:1591
assert_return(() => call($12, "load8_u", [30_844]), 0);

// memory_copy.wast:1592
assert_return(() => call($12, "load8_u", [31_043]), 0);

// memory_copy.wast:1593
assert_return(() => call($12, "load8_u", [31_242]), 0);

// memory_copy.wast:1594
assert_return(() => call($12, "load8_u", [31_441]), 0);

// memory_copy.wast:1595
assert_return(() => call($12, "load8_u", [31_640]), 0);

// memory_copy.wast:1596
assert_return(() => call($12, "load8_u", [31_839]), 0);

// memory_copy.wast:1597
assert_return(() => call($12, "load8_u", [32_038]), 0);

// memory_copy.wast:1598
assert_return(() => call($12, "load8_u", [32_237]), 0);

// memory_copy.wast:1599
assert_return(() => call($12, "load8_u", [32_436]), 0);

// memory_copy.wast:1600
assert_return(() => call($12, "load8_u", [32_635]), 0);

// memory_copy.wast:1601
assert_return(() => call($12, "load8_u", [32_834]), 0);

// memory_copy.wast:1602
assert_return(() => call($12, "load8_u", [33_033]), 0);

// memory_copy.wast:1603
assert_return(() => call($12, "load8_u", [33_232]), 0);

// memory_copy.wast:1604
assert_return(() => call($12, "load8_u", [33_431]), 0);

// memory_copy.wast:1605
assert_return(() => call($12, "load8_u", [33_630]), 0);

// memory_copy.wast:1606
assert_return(() => call($12, "load8_u", [33_829]), 0);

// memory_copy.wast:1607
assert_return(() => call($12, "load8_u", [34_028]), 0);

// memory_copy.wast:1608
assert_return(() => call($12, "load8_u", [34_227]), 0);

// memory_copy.wast:1609
assert_return(() => call($12, "load8_u", [34_426]), 0);

// memory_copy.wast:1610
assert_return(() => call($12, "load8_u", [34_625]), 0);

// memory_copy.wast:1611
assert_return(() => call($12, "load8_u", [34_824]), 0);

// memory_copy.wast:1612
assert_return(() => call($12, "load8_u", [35_023]), 0);

// memory_copy.wast:1613
assert_return(() => call($12, "load8_u", [35_222]), 0);

// memory_copy.wast:1614
assert_return(() => call($12, "load8_u", [35_421]), 0);

// memory_copy.wast:1615
assert_return(() => call($12, "load8_u", [35_620]), 0);

// memory_copy.wast:1616
assert_return(() => call($12, "load8_u", [35_819]), 0);

// memory_copy.wast:1617
assert_return(() => call($12, "load8_u", [36_018]), 0);

// memory_copy.wast:1618
assert_return(() => call($12, "load8_u", [36_217]), 0);

// memory_copy.wast:1619
assert_return(() => call($12, "load8_u", [36_416]), 0);

// memory_copy.wast:1620
assert_return(() => call($12, "load8_u", [36_615]), 0);

// memory_copy.wast:1621
assert_return(() => call($12, "load8_u", [36_814]), 0);

// memory_copy.wast:1622
assert_return(() => call($12, "load8_u", [37_013]), 0);

// memory_copy.wast:1623
assert_return(() => call($12, "load8_u", [37_212]), 0);

// memory_copy.wast:1624
assert_return(() => call($12, "load8_u", [37_411]), 0);

// memory_copy.wast:1625
assert_return(() => call($12, "load8_u", [37_610]), 0);

// memory_copy.wast:1626
assert_return(() => call($12, "load8_u", [37_809]), 0);

// memory_copy.wast:1627
assert_return(() => call($12, "load8_u", [38_008]), 0);

// memory_copy.wast:1628
assert_return(() => call($12, "load8_u", [38_207]), 0);

// memory_copy.wast:1629
assert_return(() => call($12, "load8_u", [38_406]), 0);

// memory_copy.wast:1630
assert_return(() => call($12, "load8_u", [38_605]), 0);

// memory_copy.wast:1631
assert_return(() => call($12, "load8_u", [38_804]), 0);

// memory_copy.wast:1632
assert_return(() => call($12, "load8_u", [39_003]), 0);

// memory_copy.wast:1633
assert_return(() => call($12, "load8_u", [39_202]), 0);

// memory_copy.wast:1634
assert_return(() => call($12, "load8_u", [39_401]), 0);

// memory_copy.wast:1635
assert_return(() => call($12, "load8_u", [39_600]), 0);

// memory_copy.wast:1636
assert_return(() => call($12, "load8_u", [39_799]), 0);

// memory_copy.wast:1637
assert_return(() => call($12, "load8_u", [39_998]), 0);

// memory_copy.wast:1638
assert_return(() => call($12, "load8_u", [40_197]), 0);

// memory_copy.wast:1639
assert_return(() => call($12, "load8_u", [40_396]), 0);

// memory_copy.wast:1640
assert_return(() => call($12, "load8_u", [40_595]), 0);

// memory_copy.wast:1641
assert_return(() => call($12, "load8_u", [40_794]), 0);

// memory_copy.wast:1642
assert_return(() => call($12, "load8_u", [40_993]), 0);

// memory_copy.wast:1643
assert_return(() => call($12, "load8_u", [41_192]), 0);

// memory_copy.wast:1644
assert_return(() => call($12, "load8_u", [41_391]), 0);

// memory_copy.wast:1645
assert_return(() => call($12, "load8_u", [41_590]), 0);

// memory_copy.wast:1646
assert_return(() => call($12, "load8_u", [41_789]), 0);

// memory_copy.wast:1647
assert_return(() => call($12, "load8_u", [41_988]), 0);

// memory_copy.wast:1648
assert_return(() => call($12, "load8_u", [42_187]), 0);

// memory_copy.wast:1649
assert_return(() => call($12, "load8_u", [42_386]), 0);

// memory_copy.wast:1650
assert_return(() => call($12, "load8_u", [42_585]), 0);

// memory_copy.wast:1651
assert_return(() => call($12, "load8_u", [42_784]), 0);

// memory_copy.wast:1652
assert_return(() => call($12, "load8_u", [42_983]), 0);

// memory_copy.wast:1653
assert_return(() => call($12, "load8_u", [43_182]), 0);

// memory_copy.wast:1654
assert_return(() => call($12, "load8_u", [43_381]), 0);

// memory_copy.wast:1655
assert_return(() => call($12, "load8_u", [43_580]), 0);

// memory_copy.wast:1656
assert_return(() => call($12, "load8_u", [43_779]), 0);

// memory_copy.wast:1657
assert_return(() => call($12, "load8_u", [43_978]), 0);

// memory_copy.wast:1658
assert_return(() => call($12, "load8_u", [44_177]), 0);

// memory_copy.wast:1659
assert_return(() => call($12, "load8_u", [44_376]), 0);

// memory_copy.wast:1660
assert_return(() => call($12, "load8_u", [44_575]), 0);

// memory_copy.wast:1661
assert_return(() => call($12, "load8_u", [44_774]), 0);

// memory_copy.wast:1662
assert_return(() => call($12, "load8_u", [44_973]), 0);

// memory_copy.wast:1663
assert_return(() => call($12, "load8_u", [45_172]), 0);

// memory_copy.wast:1664
assert_return(() => call($12, "load8_u", [45_371]), 0);

// memory_copy.wast:1665
assert_return(() => call($12, "load8_u", [45_570]), 0);

// memory_copy.wast:1666
assert_return(() => call($12, "load8_u", [45_769]), 0);

// memory_copy.wast:1667
assert_return(() => call($12, "load8_u", [45_968]), 0);

// memory_copy.wast:1668
assert_return(() => call($12, "load8_u", [46_167]), 0);

// memory_copy.wast:1669
assert_return(() => call($12, "load8_u", [46_366]), 0);

// memory_copy.wast:1670
assert_return(() => call($12, "load8_u", [46_565]), 0);

// memory_copy.wast:1671
assert_return(() => call($12, "load8_u", [46_764]), 0);

// memory_copy.wast:1672
assert_return(() => call($12, "load8_u", [46_963]), 0);

// memory_copy.wast:1673
assert_return(() => call($12, "load8_u", [47_162]), 0);

// memory_copy.wast:1674
assert_return(() => call($12, "load8_u", [47_361]), 0);

// memory_copy.wast:1675
assert_return(() => call($12, "load8_u", [47_560]), 0);

// memory_copy.wast:1676
assert_return(() => call($12, "load8_u", [47_759]), 0);

// memory_copy.wast:1677
assert_return(() => call($12, "load8_u", [47_958]), 0);

// memory_copy.wast:1678
assert_return(() => call($12, "load8_u", [48_157]), 0);

// memory_copy.wast:1679
assert_return(() => call($12, "load8_u", [48_356]), 0);

// memory_copy.wast:1680
assert_return(() => call($12, "load8_u", [48_555]), 0);

// memory_copy.wast:1681
assert_return(() => call($12, "load8_u", [48_754]), 0);

// memory_copy.wast:1682
assert_return(() => call($12, "load8_u", [48_953]), 0);

// memory_copy.wast:1683
assert_return(() => call($12, "load8_u", [49_152]), 0);

// memory_copy.wast:1684
assert_return(() => call($12, "load8_u", [49_351]), 0);

// memory_copy.wast:1685
assert_return(() => call($12, "load8_u", [49_550]), 0);

// memory_copy.wast:1686
assert_return(() => call($12, "load8_u", [49_749]), 0);

// memory_copy.wast:1687
assert_return(() => call($12, "load8_u", [49_948]), 0);

// memory_copy.wast:1688
assert_return(() => call($12, "load8_u", [50_147]), 0);

// memory_copy.wast:1689
assert_return(() => call($12, "load8_u", [50_346]), 0);

// memory_copy.wast:1690
assert_return(() => call($12, "load8_u", [50_545]), 0);

// memory_copy.wast:1691
assert_return(() => call($12, "load8_u", [50_744]), 0);

// memory_copy.wast:1692
assert_return(() => call($12, "load8_u", [50_943]), 0);

// memory_copy.wast:1693
assert_return(() => call($12, "load8_u", [51_142]), 0);

// memory_copy.wast:1694
assert_return(() => call($12, "load8_u", [51_341]), 0);

// memory_copy.wast:1695
assert_return(() => call($12, "load8_u", [51_540]), 0);

// memory_copy.wast:1696
assert_return(() => call($12, "load8_u", [51_739]), 0);

// memory_copy.wast:1697
assert_return(() => call($12, "load8_u", [51_938]), 0);

// memory_copy.wast:1698
assert_return(() => call($12, "load8_u", [52_137]), 0);

// memory_copy.wast:1699
assert_return(() => call($12, "load8_u", [52_336]), 0);

// memory_copy.wast:1700
assert_return(() => call($12, "load8_u", [52_535]), 0);

// memory_copy.wast:1701
assert_return(() => call($12, "load8_u", [52_734]), 0);

// memory_copy.wast:1702
assert_return(() => call($12, "load8_u", [52_933]), 0);

// memory_copy.wast:1703
assert_return(() => call($12, "load8_u", [53_132]), 0);

// memory_copy.wast:1704
assert_return(() => call($12, "load8_u", [53_331]), 0);

// memory_copy.wast:1705
assert_return(() => call($12, "load8_u", [53_530]), 0);

// memory_copy.wast:1706
assert_return(() => call($12, "load8_u", [53_729]), 0);

// memory_copy.wast:1707
assert_return(() => call($12, "load8_u", [53_928]), 0);

// memory_copy.wast:1708
assert_return(() => call($12, "load8_u", [54_127]), 0);

// memory_copy.wast:1709
assert_return(() => call($12, "load8_u", [54_326]), 0);

// memory_copy.wast:1710
assert_return(() => call($12, "load8_u", [54_525]), 0);

// memory_copy.wast:1711
assert_return(() => call($12, "load8_u", [54_724]), 0);

// memory_copy.wast:1712
assert_return(() => call($12, "load8_u", [54_923]), 0);

// memory_copy.wast:1713
assert_return(() => call($12, "load8_u", [55_122]), 0);

// memory_copy.wast:1714
assert_return(() => call($12, "load8_u", [55_321]), 0);

// memory_copy.wast:1715
assert_return(() => call($12, "load8_u", [55_520]), 0);

// memory_copy.wast:1716
assert_return(() => call($12, "load8_u", [55_719]), 0);

// memory_copy.wast:1717
assert_return(() => call($12, "load8_u", [55_918]), 0);

// memory_copy.wast:1718
assert_return(() => call($12, "load8_u", [56_117]), 0);

// memory_copy.wast:1719
assert_return(() => call($12, "load8_u", [56_316]), 0);

// memory_copy.wast:1720
assert_return(() => call($12, "load8_u", [56_515]), 0);

// memory_copy.wast:1721
assert_return(() => call($12, "load8_u", [56_714]), 0);

// memory_copy.wast:1722
assert_return(() => call($12, "load8_u", [56_913]), 0);

// memory_copy.wast:1723
assert_return(() => call($12, "load8_u", [57_112]), 0);

// memory_copy.wast:1724
assert_return(() => call($12, "load8_u", [57_311]), 0);

// memory_copy.wast:1725
assert_return(() => call($12, "load8_u", [57_510]), 0);

// memory_copy.wast:1726
assert_return(() => call($12, "load8_u", [57_709]), 0);

// memory_copy.wast:1727
assert_return(() => call($12, "load8_u", [57_908]), 0);

// memory_copy.wast:1728
assert_return(() => call($12, "load8_u", [58_107]), 0);

// memory_copy.wast:1729
assert_return(() => call($12, "load8_u", [58_306]), 0);

// memory_copy.wast:1730
assert_return(() => call($12, "load8_u", [58_505]), 0);

// memory_copy.wast:1731
assert_return(() => call($12, "load8_u", [58_704]), 0);

// memory_copy.wast:1732
assert_return(() => call($12, "load8_u", [58_903]), 0);

// memory_copy.wast:1733
assert_return(() => call($12, "load8_u", [59_102]), 0);

// memory_copy.wast:1734
assert_return(() => call($12, "load8_u", [59_301]), 0);

// memory_copy.wast:1735
assert_return(() => call($12, "load8_u", [59_500]), 0);

// memory_copy.wast:1736
assert_return(() => call($12, "load8_u", [59_699]), 0);

// memory_copy.wast:1737
assert_return(() => call($12, "load8_u", [59_898]), 0);

// memory_copy.wast:1738
assert_return(() => call($12, "load8_u", [60_097]), 0);

// memory_copy.wast:1739
assert_return(() => call($12, "load8_u", [60_296]), 0);

// memory_copy.wast:1740
assert_return(() => call($12, "load8_u", [60_495]), 0);

// memory_copy.wast:1741
assert_return(() => call($12, "load8_u", [60_694]), 0);

// memory_copy.wast:1742
assert_return(() => call($12, "load8_u", [60_893]), 0);

// memory_copy.wast:1743
assert_return(() => call($12, "load8_u", [61_092]), 0);

// memory_copy.wast:1744
assert_return(() => call($12, "load8_u", [61_291]), 0);

// memory_copy.wast:1745
assert_return(() => call($12, "load8_u", [61_490]), 0);

// memory_copy.wast:1746
assert_return(() => call($12, "load8_u", [61_689]), 0);

// memory_copy.wast:1747
assert_return(() => call($12, "load8_u", [61_888]), 0);

// memory_copy.wast:1748
assert_return(() => call($12, "load8_u", [62_087]), 0);

// memory_copy.wast:1749
assert_return(() => call($12, "load8_u", [62_286]), 0);

// memory_copy.wast:1750
assert_return(() => call($12, "load8_u", [62_485]), 0);

// memory_copy.wast:1751
assert_return(() => call($12, "load8_u", [62_684]), 0);

// memory_copy.wast:1752
assert_return(() => call($12, "load8_u", [62_883]), 0);

// memory_copy.wast:1753
assert_return(() => call($12, "load8_u", [63_082]), 0);

// memory_copy.wast:1754
assert_return(() => call($12, "load8_u", [63_281]), 0);

// memory_copy.wast:1755
assert_return(() => call($12, "load8_u", [63_480]), 0);

// memory_copy.wast:1756
assert_return(() => call($12, "load8_u", [63_679]), 0);

// memory_copy.wast:1757
assert_return(() => call($12, "load8_u", [63_878]), 0);

// memory_copy.wast:1758
assert_return(() => call($12, "load8_u", [64_077]), 0);

// memory_copy.wast:1759
assert_return(() => call($12, "load8_u", [64_276]), 0);

// memory_copy.wast:1760
assert_return(() => call($12, "load8_u", [64_475]), 0);

// memory_copy.wast:1761
assert_return(() => call($12, "load8_u", [64_674]), 0);

// memory_copy.wast:1762
assert_return(() => call($12, "load8_u", [64_873]), 0);

// memory_copy.wast:1763
assert_return(() => call($12, "load8_u", [65_072]), 0);

// memory_copy.wast:1764
assert_return(() => call($12, "load8_u", [65_271]), 0);

// memory_copy.wast:1765
assert_return(() => call($12, "load8_u", [65_470]), 0);

// memory_copy.wast:1766
assert_return(() => call($12, "load8_u", [65_515]), 0);

// memory_copy.wast:1767
assert_return(() => call($12, "load8_u", [65_516]), 1);

// memory_copy.wast:1768
assert_return(() => call($12, "load8_u", [65_517]), 2);

// memory_copy.wast:1769
assert_return(() => call($12, "load8_u", [65_518]), 3);

// memory_copy.wast:1770
assert_return(() => call($12, "load8_u", [65_519]), 4);

// memory_copy.wast:1771
assert_return(() => call($12, "load8_u", [65_520]), 5);

// memory_copy.wast:1772
assert_return(() => call($12, "load8_u", [65_521]), 6);

// memory_copy.wast:1773
assert_return(() => call($12, "load8_u", [65_522]), 7);

// memory_copy.wast:1774
assert_return(() => call($12, "load8_u", [65_523]), 8);

// memory_copy.wast:1775
assert_return(() => call($12, "load8_u", [65_524]), 9);

// memory_copy.wast:1776
assert_return(() => call($12, "load8_u", [65_525]), 10);

// memory_copy.wast:1777
assert_return(() => call($12, "load8_u", [65_526]), 11);

// memory_copy.wast:1778
assert_return(() => call($12, "load8_u", [65_527]), 12);

// memory_copy.wast:1779
assert_return(() => call($12, "load8_u", [65_528]), 13);

// memory_copy.wast:1780
assert_return(() => call($12, "load8_u", [65_529]), 14);

// memory_copy.wast:1781
assert_return(() => call($12, "load8_u", [65_530]), 15);

// memory_copy.wast:1782
assert_return(() => call($12, "load8_u", [65_531]), 16);

// memory_copy.wast:1783
assert_return(() => call($12, "load8_u", [65_532]), 17);

// memory_copy.wast:1784
assert_return(() => call($12, "load8_u", [65_533]), 18);

// memory_copy.wast:1785
assert_return(() => call($12, "load8_u", [65_534]), 19);

// memory_copy.wast:1786
assert_return(() => call($12, "load8_u", [65_535]), 20);

// memory_copy.wast:1788
let $13 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9c\x80\x80\x80\x00\x01\x00\x41\xce\xff\x03\x0b\x14\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

// memory_copy.wast:1796
assert_trap(() => call($13, "run", [65_516, 65_486, 40]));

// memory_copy.wast:1799
assert_return(() => call($13, "load8_u", [198]), 0);

// memory_copy.wast:1800
assert_return(() => call($13, "load8_u", [397]), 0);

// memory_copy.wast:1801
assert_return(() => call($13, "load8_u", [596]), 0);

// memory_copy.wast:1802
assert_return(() => call($13, "load8_u", [795]), 0);

// memory_copy.wast:1803
assert_return(() => call($13, "load8_u", [994]), 0);

// memory_copy.wast:1804
assert_return(() => call($13, "load8_u", [1_193]), 0);

// memory_copy.wast:1805
assert_return(() => call($13, "load8_u", [1_392]), 0);

// memory_copy.wast:1806
assert_return(() => call($13, "load8_u", [1_591]), 0);

// memory_copy.wast:1807
assert_return(() => call($13, "load8_u", [1_790]), 0);

// memory_copy.wast:1808
assert_return(() => call($13, "load8_u", [1_989]), 0);

// memory_copy.wast:1809
assert_return(() => call($13, "load8_u", [2_188]), 0);

// memory_copy.wast:1810
assert_return(() => call($13, "load8_u", [2_387]), 0);

// memory_copy.wast:1811
assert_return(() => call($13, "load8_u", [2_586]), 0);

// memory_copy.wast:1812
assert_return(() => call($13, "load8_u", [2_785]), 0);

// memory_copy.wast:1813
assert_return(() => call($13, "load8_u", [2_984]), 0);

// memory_copy.wast:1814
assert_return(() => call($13, "load8_u", [3_183]), 0);

// memory_copy.wast:1815
assert_return(() => call($13, "load8_u", [3_382]), 0);

// memory_copy.wast:1816
assert_return(() => call($13, "load8_u", [3_581]), 0);

// memory_copy.wast:1817
assert_return(() => call($13, "load8_u", [3_780]), 0);

// memory_copy.wast:1818
assert_return(() => call($13, "load8_u", [3_979]), 0);

// memory_copy.wast:1819
assert_return(() => call($13, "load8_u", [4_178]), 0);

// memory_copy.wast:1820
assert_return(() => call($13, "load8_u", [4_377]), 0);

// memory_copy.wast:1821
assert_return(() => call($13, "load8_u", [4_576]), 0);

// memory_copy.wast:1822
assert_return(() => call($13, "load8_u", [4_775]), 0);

// memory_copy.wast:1823
assert_return(() => call($13, "load8_u", [4_974]), 0);

// memory_copy.wast:1824
assert_return(() => call($13, "load8_u", [5_173]), 0);

// memory_copy.wast:1825
assert_return(() => call($13, "load8_u", [5_372]), 0);

// memory_copy.wast:1826
assert_return(() => call($13, "load8_u", [5_571]), 0);

// memory_copy.wast:1827
assert_return(() => call($13, "load8_u", [5_770]), 0);

// memory_copy.wast:1828
assert_return(() => call($13, "load8_u", [5_969]), 0);

// memory_copy.wast:1829
assert_return(() => call($13, "load8_u", [6_168]), 0);

// memory_copy.wast:1830
assert_return(() => call($13, "load8_u", [6_367]), 0);

// memory_copy.wast:1831
assert_return(() => call($13, "load8_u", [6_566]), 0);

// memory_copy.wast:1832
assert_return(() => call($13, "load8_u", [6_765]), 0);

// memory_copy.wast:1833
assert_return(() => call($13, "load8_u", [6_964]), 0);

// memory_copy.wast:1834
assert_return(() => call($13, "load8_u", [7_163]), 0);

// memory_copy.wast:1835
assert_return(() => call($13, "load8_u", [7_362]), 0);

// memory_copy.wast:1836
assert_return(() => call($13, "load8_u", [7_561]), 0);

// memory_copy.wast:1837
assert_return(() => call($13, "load8_u", [7_760]), 0);

// memory_copy.wast:1838
assert_return(() => call($13, "load8_u", [7_959]), 0);

// memory_copy.wast:1839
assert_return(() => call($13, "load8_u", [8_158]), 0);

// memory_copy.wast:1840
assert_return(() => call($13, "load8_u", [8_357]), 0);

// memory_copy.wast:1841
assert_return(() => call($13, "load8_u", [8_556]), 0);

// memory_copy.wast:1842
assert_return(() => call($13, "load8_u", [8_755]), 0);

// memory_copy.wast:1843
assert_return(() => call($13, "load8_u", [8_954]), 0);

// memory_copy.wast:1844
assert_return(() => call($13, "load8_u", [9_153]), 0);

// memory_copy.wast:1845
assert_return(() => call($13, "load8_u", [9_352]), 0);

// memory_copy.wast:1846
assert_return(() => call($13, "load8_u", [9_551]), 0);

// memory_copy.wast:1847
assert_return(() => call($13, "load8_u", [9_750]), 0);

// memory_copy.wast:1848
assert_return(() => call($13, "load8_u", [9_949]), 0);

// memory_copy.wast:1849
assert_return(() => call($13, "load8_u", [10_148]), 0);

// memory_copy.wast:1850
assert_return(() => call($13, "load8_u", [10_347]), 0);

// memory_copy.wast:1851
assert_return(() => call($13, "load8_u", [10_546]), 0);

// memory_copy.wast:1852
assert_return(() => call($13, "load8_u", [10_745]), 0);

// memory_copy.wast:1853
assert_return(() => call($13, "load8_u", [10_944]), 0);

// memory_copy.wast:1854
assert_return(() => call($13, "load8_u", [11_143]), 0);

// memory_copy.wast:1855
assert_return(() => call($13, "load8_u", [11_342]), 0);

// memory_copy.wast:1856
assert_return(() => call($13, "load8_u", [11_541]), 0);

// memory_copy.wast:1857
assert_return(() => call($13, "load8_u", [11_740]), 0);

// memory_copy.wast:1858
assert_return(() => call($13, "load8_u", [11_939]), 0);

// memory_copy.wast:1859
assert_return(() => call($13, "load8_u", [12_138]), 0);

// memory_copy.wast:1860
assert_return(() => call($13, "load8_u", [12_337]), 0);

// memory_copy.wast:1861
assert_return(() => call($13, "load8_u", [12_536]), 0);

// memory_copy.wast:1862
assert_return(() => call($13, "load8_u", [12_735]), 0);

// memory_copy.wast:1863
assert_return(() => call($13, "load8_u", [12_934]), 0);

// memory_copy.wast:1864
assert_return(() => call($13, "load8_u", [13_133]), 0);

// memory_copy.wast:1865
assert_return(() => call($13, "load8_u", [13_332]), 0);

// memory_copy.wast:1866
assert_return(() => call($13, "load8_u", [13_531]), 0);

// memory_copy.wast:1867
assert_return(() => call($13, "load8_u", [13_730]), 0);

// memory_copy.wast:1868
assert_return(() => call($13, "load8_u", [13_929]), 0);

// memory_copy.wast:1869
assert_return(() => call($13, "load8_u", [14_128]), 0);

// memory_copy.wast:1870
assert_return(() => call($13, "load8_u", [14_327]), 0);

// memory_copy.wast:1871
assert_return(() => call($13, "load8_u", [14_526]), 0);

// memory_copy.wast:1872
assert_return(() => call($13, "load8_u", [14_725]), 0);

// memory_copy.wast:1873
assert_return(() => call($13, "load8_u", [14_924]), 0);

// memory_copy.wast:1874
assert_return(() => call($13, "load8_u", [15_123]), 0);

// memory_copy.wast:1875
assert_return(() => call($13, "load8_u", [15_322]), 0);

// memory_copy.wast:1876
assert_return(() => call($13, "load8_u", [15_521]), 0);

// memory_copy.wast:1877
assert_return(() => call($13, "load8_u", [15_720]), 0);

// memory_copy.wast:1878
assert_return(() => call($13, "load8_u", [15_919]), 0);

// memory_copy.wast:1879
assert_return(() => call($13, "load8_u", [16_118]), 0);

// memory_copy.wast:1880
assert_return(() => call($13, "load8_u", [16_317]), 0);

// memory_copy.wast:1881
assert_return(() => call($13, "load8_u", [16_516]), 0);

// memory_copy.wast:1882
assert_return(() => call($13, "load8_u", [16_715]), 0);

// memory_copy.wast:1883
assert_return(() => call($13, "load8_u", [16_914]), 0);

// memory_copy.wast:1884
assert_return(() => call($13, "load8_u", [17_113]), 0);

// memory_copy.wast:1885
assert_return(() => call($13, "load8_u", [17_312]), 0);

// memory_copy.wast:1886
assert_return(() => call($13, "load8_u", [17_511]), 0);

// memory_copy.wast:1887
assert_return(() => call($13, "load8_u", [17_710]), 0);

// memory_copy.wast:1888
assert_return(() => call($13, "load8_u", [17_909]), 0);

// memory_copy.wast:1889
assert_return(() => call($13, "load8_u", [18_108]), 0);

// memory_copy.wast:1890
assert_return(() => call($13, "load8_u", [18_307]), 0);

// memory_copy.wast:1891
assert_return(() => call($13, "load8_u", [18_506]), 0);

// memory_copy.wast:1892
assert_return(() => call($13, "load8_u", [18_705]), 0);

// memory_copy.wast:1893
assert_return(() => call($13, "load8_u", [18_904]), 0);

// memory_copy.wast:1894
assert_return(() => call($13, "load8_u", [19_103]), 0);

// memory_copy.wast:1895
assert_return(() => call($13, "load8_u", [19_302]), 0);

// memory_copy.wast:1896
assert_return(() => call($13, "load8_u", [19_501]), 0);

// memory_copy.wast:1897
assert_return(() => call($13, "load8_u", [19_700]), 0);

// memory_copy.wast:1898
assert_return(() => call($13, "load8_u", [19_899]), 0);

// memory_copy.wast:1899
assert_return(() => call($13, "load8_u", [20_098]), 0);

// memory_copy.wast:1900
assert_return(() => call($13, "load8_u", [20_297]), 0);

// memory_copy.wast:1901
assert_return(() => call($13, "load8_u", [20_496]), 0);

// memory_copy.wast:1902
assert_return(() => call($13, "load8_u", [20_695]), 0);

// memory_copy.wast:1903
assert_return(() => call($13, "load8_u", [20_894]), 0);

// memory_copy.wast:1904
assert_return(() => call($13, "load8_u", [21_093]), 0);

// memory_copy.wast:1905
assert_return(() => call($13, "load8_u", [21_292]), 0);

// memory_copy.wast:1906
assert_return(() => call($13, "load8_u", [21_491]), 0);

// memory_copy.wast:1907
assert_return(() => call($13, "load8_u", [21_690]), 0);

// memory_copy.wast:1908
assert_return(() => call($13, "load8_u", [21_889]), 0);

// memory_copy.wast:1909
assert_return(() => call($13, "load8_u", [22_088]), 0);

// memory_copy.wast:1910
assert_return(() => call($13, "load8_u", [22_287]), 0);

// memory_copy.wast:1911
assert_return(() => call($13, "load8_u", [22_486]), 0);

// memory_copy.wast:1912
assert_return(() => call($13, "load8_u", [22_685]), 0);

// memory_copy.wast:1913
assert_return(() => call($13, "load8_u", [22_884]), 0);

// memory_copy.wast:1914
assert_return(() => call($13, "load8_u", [23_083]), 0);

// memory_copy.wast:1915
assert_return(() => call($13, "load8_u", [23_282]), 0);

// memory_copy.wast:1916
assert_return(() => call($13, "load8_u", [23_481]), 0);

// memory_copy.wast:1917
assert_return(() => call($13, "load8_u", [23_680]), 0);

// memory_copy.wast:1918
assert_return(() => call($13, "load8_u", [23_879]), 0);

// memory_copy.wast:1919
assert_return(() => call($13, "load8_u", [24_078]), 0);

// memory_copy.wast:1920
assert_return(() => call($13, "load8_u", [24_277]), 0);

// memory_copy.wast:1921
assert_return(() => call($13, "load8_u", [24_476]), 0);

// memory_copy.wast:1922
assert_return(() => call($13, "load8_u", [24_675]), 0);

// memory_copy.wast:1923
assert_return(() => call($13, "load8_u", [24_874]), 0);

// memory_copy.wast:1924
assert_return(() => call($13, "load8_u", [25_073]), 0);

// memory_copy.wast:1925
assert_return(() => call($13, "load8_u", [25_272]), 0);

// memory_copy.wast:1926
assert_return(() => call($13, "load8_u", [25_471]), 0);

// memory_copy.wast:1927
assert_return(() => call($13, "load8_u", [25_670]), 0);

// memory_copy.wast:1928
assert_return(() => call($13, "load8_u", [25_869]), 0);

// memory_copy.wast:1929
assert_return(() => call($13, "load8_u", [26_068]), 0);

// memory_copy.wast:1930
assert_return(() => call($13, "load8_u", [26_267]), 0);

// memory_copy.wast:1931
assert_return(() => call($13, "load8_u", [26_466]), 0);

// memory_copy.wast:1932
assert_return(() => call($13, "load8_u", [26_665]), 0);

// memory_copy.wast:1933
assert_return(() => call($13, "load8_u", [26_864]), 0);

// memory_copy.wast:1934
assert_return(() => call($13, "load8_u", [27_063]), 0);

// memory_copy.wast:1935
assert_return(() => call($13, "load8_u", [27_262]), 0);

// memory_copy.wast:1936
assert_return(() => call($13, "load8_u", [27_461]), 0);

// memory_copy.wast:1937
assert_return(() => call($13, "load8_u", [27_660]), 0);

// memory_copy.wast:1938
assert_return(() => call($13, "load8_u", [27_859]), 0);

// memory_copy.wast:1939
assert_return(() => call($13, "load8_u", [28_058]), 0);

// memory_copy.wast:1940
assert_return(() => call($13, "load8_u", [28_257]), 0);

// memory_copy.wast:1941
assert_return(() => call($13, "load8_u", [28_456]), 0);

// memory_copy.wast:1942
assert_return(() => call($13, "load8_u", [28_655]), 0);

// memory_copy.wast:1943
assert_return(() => call($13, "load8_u", [28_854]), 0);

// memory_copy.wast:1944
assert_return(() => call($13, "load8_u", [29_053]), 0);

// memory_copy.wast:1945
assert_return(() => call($13, "load8_u", [29_252]), 0);

// memory_copy.wast:1946
assert_return(() => call($13, "load8_u", [29_451]), 0);

// memory_copy.wast:1947
assert_return(() => call($13, "load8_u", [29_650]), 0);

// memory_copy.wast:1948
assert_return(() => call($13, "load8_u", [29_849]), 0);

// memory_copy.wast:1949
assert_return(() => call($13, "load8_u", [30_048]), 0);

// memory_copy.wast:1950
assert_return(() => call($13, "load8_u", [30_247]), 0);

// memory_copy.wast:1951
assert_return(() => call($13, "load8_u", [30_446]), 0);

// memory_copy.wast:1952
assert_return(() => call($13, "load8_u", [30_645]), 0);

// memory_copy.wast:1953
assert_return(() => call($13, "load8_u", [30_844]), 0);

// memory_copy.wast:1954
assert_return(() => call($13, "load8_u", [31_043]), 0);

// memory_copy.wast:1955
assert_return(() => call($13, "load8_u", [31_242]), 0);

// memory_copy.wast:1956
assert_return(() => call($13, "load8_u", [31_441]), 0);

// memory_copy.wast:1957
assert_return(() => call($13, "load8_u", [31_640]), 0);

// memory_copy.wast:1958
assert_return(() => call($13, "load8_u", [31_839]), 0);

// memory_copy.wast:1959
assert_return(() => call($13, "load8_u", [32_038]), 0);

// memory_copy.wast:1960
assert_return(() => call($13, "load8_u", [32_237]), 0);

// memory_copy.wast:1961
assert_return(() => call($13, "load8_u", [32_436]), 0);

// memory_copy.wast:1962
assert_return(() => call($13, "load8_u", [32_635]), 0);

// memory_copy.wast:1963
assert_return(() => call($13, "load8_u", [32_834]), 0);

// memory_copy.wast:1964
assert_return(() => call($13, "load8_u", [33_033]), 0);

// memory_copy.wast:1965
assert_return(() => call($13, "load8_u", [33_232]), 0);

// memory_copy.wast:1966
assert_return(() => call($13, "load8_u", [33_431]), 0);

// memory_copy.wast:1967
assert_return(() => call($13, "load8_u", [33_630]), 0);

// memory_copy.wast:1968
assert_return(() => call($13, "load8_u", [33_829]), 0);

// memory_copy.wast:1969
assert_return(() => call($13, "load8_u", [34_028]), 0);

// memory_copy.wast:1970
assert_return(() => call($13, "load8_u", [34_227]), 0);

// memory_copy.wast:1971
assert_return(() => call($13, "load8_u", [34_426]), 0);

// memory_copy.wast:1972
assert_return(() => call($13, "load8_u", [34_625]), 0);

// memory_copy.wast:1973
assert_return(() => call($13, "load8_u", [34_824]), 0);

// memory_copy.wast:1974
assert_return(() => call($13, "load8_u", [35_023]), 0);

// memory_copy.wast:1975
assert_return(() => call($13, "load8_u", [35_222]), 0);

// memory_copy.wast:1976
assert_return(() => call($13, "load8_u", [35_421]), 0);

// memory_copy.wast:1977
assert_return(() => call($13, "load8_u", [35_620]), 0);

// memory_copy.wast:1978
assert_return(() => call($13, "load8_u", [35_819]), 0);

// memory_copy.wast:1979
assert_return(() => call($13, "load8_u", [36_018]), 0);

// memory_copy.wast:1980
assert_return(() => call($13, "load8_u", [36_217]), 0);

// memory_copy.wast:1981
assert_return(() => call($13, "load8_u", [36_416]), 0);

// memory_copy.wast:1982
assert_return(() => call($13, "load8_u", [36_615]), 0);

// memory_copy.wast:1983
assert_return(() => call($13, "load8_u", [36_814]), 0);

// memory_copy.wast:1984
assert_return(() => call($13, "load8_u", [37_013]), 0);

// memory_copy.wast:1985
assert_return(() => call($13, "load8_u", [37_212]), 0);

// memory_copy.wast:1986
assert_return(() => call($13, "load8_u", [37_411]), 0);

// memory_copy.wast:1987
assert_return(() => call($13, "load8_u", [37_610]), 0);

// memory_copy.wast:1988
assert_return(() => call($13, "load8_u", [37_809]), 0);

// memory_copy.wast:1989
assert_return(() => call($13, "load8_u", [38_008]), 0);

// memory_copy.wast:1990
assert_return(() => call($13, "load8_u", [38_207]), 0);

// memory_copy.wast:1991
assert_return(() => call($13, "load8_u", [38_406]), 0);

// memory_copy.wast:1992
assert_return(() => call($13, "load8_u", [38_605]), 0);

// memory_copy.wast:1993
assert_return(() => call($13, "load8_u", [38_804]), 0);

// memory_copy.wast:1994
assert_return(() => call($13, "load8_u", [39_003]), 0);

// memory_copy.wast:1995
assert_return(() => call($13, "load8_u", [39_202]), 0);

// memory_copy.wast:1996
assert_return(() => call($13, "load8_u", [39_401]), 0);

// memory_copy.wast:1997
assert_return(() => call($13, "load8_u", [39_600]), 0);

// memory_copy.wast:1998
assert_return(() => call($13, "load8_u", [39_799]), 0);

// memory_copy.wast:1999
assert_return(() => call($13, "load8_u", [39_998]), 0);

// memory_copy.wast:2000
assert_return(() => call($13, "load8_u", [40_197]), 0);

// memory_copy.wast:2001
assert_return(() => call($13, "load8_u", [40_396]), 0);

// memory_copy.wast:2002
assert_return(() => call($13, "load8_u", [40_595]), 0);

// memory_copy.wast:2003
assert_return(() => call($13, "load8_u", [40_794]), 0);

// memory_copy.wast:2004
assert_return(() => call($13, "load8_u", [40_993]), 0);

// memory_copy.wast:2005
assert_return(() => call($13, "load8_u", [41_192]), 0);

// memory_copy.wast:2006
assert_return(() => call($13, "load8_u", [41_391]), 0);

// memory_copy.wast:2007
assert_return(() => call($13, "load8_u", [41_590]), 0);

// memory_copy.wast:2008
assert_return(() => call($13, "load8_u", [41_789]), 0);

// memory_copy.wast:2009
assert_return(() => call($13, "load8_u", [41_988]), 0);

// memory_copy.wast:2010
assert_return(() => call($13, "load8_u", [42_187]), 0);

// memory_copy.wast:2011
assert_return(() => call($13, "load8_u", [42_386]), 0);

// memory_copy.wast:2012
assert_return(() => call($13, "load8_u", [42_585]), 0);

// memory_copy.wast:2013
assert_return(() => call($13, "load8_u", [42_784]), 0);

// memory_copy.wast:2014
assert_return(() => call($13, "load8_u", [42_983]), 0);

// memory_copy.wast:2015
assert_return(() => call($13, "load8_u", [43_182]), 0);

// memory_copy.wast:2016
assert_return(() => call($13, "load8_u", [43_381]), 0);

// memory_copy.wast:2017
assert_return(() => call($13, "load8_u", [43_580]), 0);

// memory_copy.wast:2018
assert_return(() => call($13, "load8_u", [43_779]), 0);

// memory_copy.wast:2019
assert_return(() => call($13, "load8_u", [43_978]), 0);

// memory_copy.wast:2020
assert_return(() => call($13, "load8_u", [44_177]), 0);

// memory_copy.wast:2021
assert_return(() => call($13, "load8_u", [44_376]), 0);

// memory_copy.wast:2022
assert_return(() => call($13, "load8_u", [44_575]), 0);

// memory_copy.wast:2023
assert_return(() => call($13, "load8_u", [44_774]), 0);

// memory_copy.wast:2024
assert_return(() => call($13, "load8_u", [44_973]), 0);

// memory_copy.wast:2025
assert_return(() => call($13, "load8_u", [45_172]), 0);

// memory_copy.wast:2026
assert_return(() => call($13, "load8_u", [45_371]), 0);

// memory_copy.wast:2027
assert_return(() => call($13, "load8_u", [45_570]), 0);

// memory_copy.wast:2028
assert_return(() => call($13, "load8_u", [45_769]), 0);

// memory_copy.wast:2029
assert_return(() => call($13, "load8_u", [45_968]), 0);

// memory_copy.wast:2030
assert_return(() => call($13, "load8_u", [46_167]), 0);

// memory_copy.wast:2031
assert_return(() => call($13, "load8_u", [46_366]), 0);

// memory_copy.wast:2032
assert_return(() => call($13, "load8_u", [46_565]), 0);

// memory_copy.wast:2033
assert_return(() => call($13, "load8_u", [46_764]), 0);

// memory_copy.wast:2034
assert_return(() => call($13, "load8_u", [46_963]), 0);

// memory_copy.wast:2035
assert_return(() => call($13, "load8_u", [47_162]), 0);

// memory_copy.wast:2036
assert_return(() => call($13, "load8_u", [47_361]), 0);

// memory_copy.wast:2037
assert_return(() => call($13, "load8_u", [47_560]), 0);

// memory_copy.wast:2038
assert_return(() => call($13, "load8_u", [47_759]), 0);

// memory_copy.wast:2039
assert_return(() => call($13, "load8_u", [47_958]), 0);

// memory_copy.wast:2040
assert_return(() => call($13, "load8_u", [48_157]), 0);

// memory_copy.wast:2041
assert_return(() => call($13, "load8_u", [48_356]), 0);

// memory_copy.wast:2042
assert_return(() => call($13, "load8_u", [48_555]), 0);

// memory_copy.wast:2043
assert_return(() => call($13, "load8_u", [48_754]), 0);

// memory_copy.wast:2044
assert_return(() => call($13, "load8_u", [48_953]), 0);

// memory_copy.wast:2045
assert_return(() => call($13, "load8_u", [49_152]), 0);

// memory_copy.wast:2046
assert_return(() => call($13, "load8_u", [49_351]), 0);

// memory_copy.wast:2047
assert_return(() => call($13, "load8_u", [49_550]), 0);

// memory_copy.wast:2048
assert_return(() => call($13, "load8_u", [49_749]), 0);

// memory_copy.wast:2049
assert_return(() => call($13, "load8_u", [49_948]), 0);

// memory_copy.wast:2050
assert_return(() => call($13, "load8_u", [50_147]), 0);

// memory_copy.wast:2051
assert_return(() => call($13, "load8_u", [50_346]), 0);

// memory_copy.wast:2052
assert_return(() => call($13, "load8_u", [50_545]), 0);

// memory_copy.wast:2053
assert_return(() => call($13, "load8_u", [50_744]), 0);

// memory_copy.wast:2054
assert_return(() => call($13, "load8_u", [50_943]), 0);

// memory_copy.wast:2055
assert_return(() => call($13, "load8_u", [51_142]), 0);

// memory_copy.wast:2056
assert_return(() => call($13, "load8_u", [51_341]), 0);

// memory_copy.wast:2057
assert_return(() => call($13, "load8_u", [51_540]), 0);

// memory_copy.wast:2058
assert_return(() => call($13, "load8_u", [51_739]), 0);

// memory_copy.wast:2059
assert_return(() => call($13, "load8_u", [51_938]), 0);

// memory_copy.wast:2060
assert_return(() => call($13, "load8_u", [52_137]), 0);

// memory_copy.wast:2061
assert_return(() => call($13, "load8_u", [52_336]), 0);

// memory_copy.wast:2062
assert_return(() => call($13, "load8_u", [52_535]), 0);

// memory_copy.wast:2063
assert_return(() => call($13, "load8_u", [52_734]), 0);

// memory_copy.wast:2064
assert_return(() => call($13, "load8_u", [52_933]), 0);

// memory_copy.wast:2065
assert_return(() => call($13, "load8_u", [53_132]), 0);

// memory_copy.wast:2066
assert_return(() => call($13, "load8_u", [53_331]), 0);

// memory_copy.wast:2067
assert_return(() => call($13, "load8_u", [53_530]), 0);

// memory_copy.wast:2068
assert_return(() => call($13, "load8_u", [53_729]), 0);

// memory_copy.wast:2069
assert_return(() => call($13, "load8_u", [53_928]), 0);

// memory_copy.wast:2070
assert_return(() => call($13, "load8_u", [54_127]), 0);

// memory_copy.wast:2071
assert_return(() => call($13, "load8_u", [54_326]), 0);

// memory_copy.wast:2072
assert_return(() => call($13, "load8_u", [54_525]), 0);

// memory_copy.wast:2073
assert_return(() => call($13, "load8_u", [54_724]), 0);

// memory_copy.wast:2074
assert_return(() => call($13, "load8_u", [54_923]), 0);

// memory_copy.wast:2075
assert_return(() => call($13, "load8_u", [55_122]), 0);

// memory_copy.wast:2076
assert_return(() => call($13, "load8_u", [55_321]), 0);

// memory_copy.wast:2077
assert_return(() => call($13, "load8_u", [55_520]), 0);

// memory_copy.wast:2078
assert_return(() => call($13, "load8_u", [55_719]), 0);

// memory_copy.wast:2079
assert_return(() => call($13, "load8_u", [55_918]), 0);

// memory_copy.wast:2080
assert_return(() => call($13, "load8_u", [56_117]), 0);

// memory_copy.wast:2081
assert_return(() => call($13, "load8_u", [56_316]), 0);

// memory_copy.wast:2082
assert_return(() => call($13, "load8_u", [56_515]), 0);

// memory_copy.wast:2083
assert_return(() => call($13, "load8_u", [56_714]), 0);

// memory_copy.wast:2084
assert_return(() => call($13, "load8_u", [56_913]), 0);

// memory_copy.wast:2085
assert_return(() => call($13, "load8_u", [57_112]), 0);

// memory_copy.wast:2086
assert_return(() => call($13, "load8_u", [57_311]), 0);

// memory_copy.wast:2087
assert_return(() => call($13, "load8_u", [57_510]), 0);

// memory_copy.wast:2088
assert_return(() => call($13, "load8_u", [57_709]), 0);

// memory_copy.wast:2089
assert_return(() => call($13, "load8_u", [57_908]), 0);

// memory_copy.wast:2090
assert_return(() => call($13, "load8_u", [58_107]), 0);

// memory_copy.wast:2091
assert_return(() => call($13, "load8_u", [58_306]), 0);

// memory_copy.wast:2092
assert_return(() => call($13, "load8_u", [58_505]), 0);

// memory_copy.wast:2093
assert_return(() => call($13, "load8_u", [58_704]), 0);

// memory_copy.wast:2094
assert_return(() => call($13, "load8_u", [58_903]), 0);

// memory_copy.wast:2095
assert_return(() => call($13, "load8_u", [59_102]), 0);

// memory_copy.wast:2096
assert_return(() => call($13, "load8_u", [59_301]), 0);

// memory_copy.wast:2097
assert_return(() => call($13, "load8_u", [59_500]), 0);

// memory_copy.wast:2098
assert_return(() => call($13, "load8_u", [59_699]), 0);

// memory_copy.wast:2099
assert_return(() => call($13, "load8_u", [59_898]), 0);

// memory_copy.wast:2100
assert_return(() => call($13, "load8_u", [60_097]), 0);

// memory_copy.wast:2101
assert_return(() => call($13, "load8_u", [60_296]), 0);

// memory_copy.wast:2102
assert_return(() => call($13, "load8_u", [60_495]), 0);

// memory_copy.wast:2103
assert_return(() => call($13, "load8_u", [60_694]), 0);

// memory_copy.wast:2104
assert_return(() => call($13, "load8_u", [60_893]), 0);

// memory_copy.wast:2105
assert_return(() => call($13, "load8_u", [61_092]), 0);

// memory_copy.wast:2106
assert_return(() => call($13, "load8_u", [61_291]), 0);

// memory_copy.wast:2107
assert_return(() => call($13, "load8_u", [61_490]), 0);

// memory_copy.wast:2108
assert_return(() => call($13, "load8_u", [61_689]), 0);

// memory_copy.wast:2109
assert_return(() => call($13, "load8_u", [61_888]), 0);

// memory_copy.wast:2110
assert_return(() => call($13, "load8_u", [62_087]), 0);

// memory_copy.wast:2111
assert_return(() => call($13, "load8_u", [62_286]), 0);

// memory_copy.wast:2112
assert_return(() => call($13, "load8_u", [62_485]), 0);

// memory_copy.wast:2113
assert_return(() => call($13, "load8_u", [62_684]), 0);

// memory_copy.wast:2114
assert_return(() => call($13, "load8_u", [62_883]), 0);

// memory_copy.wast:2115
assert_return(() => call($13, "load8_u", [63_082]), 0);

// memory_copy.wast:2116
assert_return(() => call($13, "load8_u", [63_281]), 0);

// memory_copy.wast:2117
assert_return(() => call($13, "load8_u", [63_480]), 0);

// memory_copy.wast:2118
assert_return(() => call($13, "load8_u", [63_679]), 0);

// memory_copy.wast:2119
assert_return(() => call($13, "load8_u", [63_878]), 0);

// memory_copy.wast:2120
assert_return(() => call($13, "load8_u", [64_077]), 0);

// memory_copy.wast:2121
assert_return(() => call($13, "load8_u", [64_276]), 0);

// memory_copy.wast:2122
assert_return(() => call($13, "load8_u", [64_475]), 0);

// memory_copy.wast:2123
assert_return(() => call($13, "load8_u", [64_674]), 0);

// memory_copy.wast:2124
assert_return(() => call($13, "load8_u", [64_873]), 0);

// memory_copy.wast:2125
assert_return(() => call($13, "load8_u", [65_072]), 0);

// memory_copy.wast:2126
assert_return(() => call($13, "load8_u", [65_271]), 0);

// memory_copy.wast:2127
assert_return(() => call($13, "load8_u", [65_470]), 0);

// memory_copy.wast:2128
assert_return(() => call($13, "load8_u", [65_486]), 0);

// memory_copy.wast:2129
assert_return(() => call($13, "load8_u", [65_487]), 1);

// memory_copy.wast:2130
assert_return(() => call($13, "load8_u", [65_488]), 2);

// memory_copy.wast:2131
assert_return(() => call($13, "load8_u", [65_489]), 3);

// memory_copy.wast:2132
assert_return(() => call($13, "load8_u", [65_490]), 4);

// memory_copy.wast:2133
assert_return(() => call($13, "load8_u", [65_491]), 5);

// memory_copy.wast:2134
assert_return(() => call($13, "load8_u", [65_492]), 6);

// memory_copy.wast:2135
assert_return(() => call($13, "load8_u", [65_493]), 7);

// memory_copy.wast:2136
assert_return(() => call($13, "load8_u", [65_494]), 8);

// memory_copy.wast:2137
assert_return(() => call($13, "load8_u", [65_495]), 9);

// memory_copy.wast:2138
assert_return(() => call($13, "load8_u", [65_496]), 10);

// memory_copy.wast:2139
assert_return(() => call($13, "load8_u", [65_497]), 11);

// memory_copy.wast:2140
assert_return(() => call($13, "load8_u", [65_498]), 12);

// memory_copy.wast:2141
assert_return(() => call($13, "load8_u", [65_499]), 13);

// memory_copy.wast:2142
assert_return(() => call($13, "load8_u", [65_500]), 14);

// memory_copy.wast:2143
assert_return(() => call($13, "load8_u", [65_501]), 15);

// memory_copy.wast:2144
assert_return(() => call($13, "load8_u", [65_502]), 16);

// memory_copy.wast:2145
assert_return(() => call($13, "load8_u", [65_503]), 17);

// memory_copy.wast:2146
assert_return(() => call($13, "load8_u", [65_504]), 18);

// memory_copy.wast:2147
assert_return(() => call($13, "load8_u", [65_505]), 19);

// memory_copy.wast:2149
let $14 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9c\x80\x80\x80\x00\x01\x00\x41\xec\xff\x03\x0b\x14\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

// memory_copy.wast:2157
assert_trap(() => call($14, "run", [65_486, 65_516, 40]));

// memory_copy.wast:2160
assert_return(() => call($14, "load8_u", [198]), 0);

// memory_copy.wast:2161
assert_return(() => call($14, "load8_u", [397]), 0);

// memory_copy.wast:2162
assert_return(() => call($14, "load8_u", [596]), 0);

// memory_copy.wast:2163
assert_return(() => call($14, "load8_u", [795]), 0);

// memory_copy.wast:2164
assert_return(() => call($14, "load8_u", [994]), 0);

// memory_copy.wast:2165
assert_return(() => call($14, "load8_u", [1_193]), 0);

// memory_copy.wast:2166
assert_return(() => call($14, "load8_u", [1_392]), 0);

// memory_copy.wast:2167
assert_return(() => call($14, "load8_u", [1_591]), 0);

// memory_copy.wast:2168
assert_return(() => call($14, "load8_u", [1_790]), 0);

// memory_copy.wast:2169
assert_return(() => call($14, "load8_u", [1_989]), 0);

// memory_copy.wast:2170
assert_return(() => call($14, "load8_u", [2_188]), 0);

// memory_copy.wast:2171
assert_return(() => call($14, "load8_u", [2_387]), 0);

// memory_copy.wast:2172
assert_return(() => call($14, "load8_u", [2_586]), 0);

// memory_copy.wast:2173
assert_return(() => call($14, "load8_u", [2_785]), 0);

// memory_copy.wast:2174
assert_return(() => call($14, "load8_u", [2_984]), 0);

// memory_copy.wast:2175
assert_return(() => call($14, "load8_u", [3_183]), 0);

// memory_copy.wast:2176
assert_return(() => call($14, "load8_u", [3_382]), 0);

// memory_copy.wast:2177
assert_return(() => call($14, "load8_u", [3_581]), 0);

// memory_copy.wast:2178
assert_return(() => call($14, "load8_u", [3_780]), 0);

// memory_copy.wast:2179
assert_return(() => call($14, "load8_u", [3_979]), 0);

// memory_copy.wast:2180
assert_return(() => call($14, "load8_u", [4_178]), 0);

// memory_copy.wast:2181
assert_return(() => call($14, "load8_u", [4_377]), 0);

// memory_copy.wast:2182
assert_return(() => call($14, "load8_u", [4_576]), 0);

// memory_copy.wast:2183
assert_return(() => call($14, "load8_u", [4_775]), 0);

// memory_copy.wast:2184
assert_return(() => call($14, "load8_u", [4_974]), 0);

// memory_copy.wast:2185
assert_return(() => call($14, "load8_u", [5_173]), 0);

// memory_copy.wast:2186
assert_return(() => call($14, "load8_u", [5_372]), 0);

// memory_copy.wast:2187
assert_return(() => call($14, "load8_u", [5_571]), 0);

// memory_copy.wast:2188
assert_return(() => call($14, "load8_u", [5_770]), 0);

// memory_copy.wast:2189
assert_return(() => call($14, "load8_u", [5_969]), 0);

// memory_copy.wast:2190
assert_return(() => call($14, "load8_u", [6_168]), 0);

// memory_copy.wast:2191
assert_return(() => call($14, "load8_u", [6_367]), 0);

// memory_copy.wast:2192
assert_return(() => call($14, "load8_u", [6_566]), 0);

// memory_copy.wast:2193
assert_return(() => call($14, "load8_u", [6_765]), 0);

// memory_copy.wast:2194
assert_return(() => call($14, "load8_u", [6_964]), 0);

// memory_copy.wast:2195
assert_return(() => call($14, "load8_u", [7_163]), 0);

// memory_copy.wast:2196
assert_return(() => call($14, "load8_u", [7_362]), 0);

// memory_copy.wast:2197
assert_return(() => call($14, "load8_u", [7_561]), 0);

// memory_copy.wast:2198
assert_return(() => call($14, "load8_u", [7_760]), 0);

// memory_copy.wast:2199
assert_return(() => call($14, "load8_u", [7_959]), 0);

// memory_copy.wast:2200
assert_return(() => call($14, "load8_u", [8_158]), 0);

// memory_copy.wast:2201
assert_return(() => call($14, "load8_u", [8_357]), 0);

// memory_copy.wast:2202
assert_return(() => call($14, "load8_u", [8_556]), 0);

// memory_copy.wast:2203
assert_return(() => call($14, "load8_u", [8_755]), 0);

// memory_copy.wast:2204
assert_return(() => call($14, "load8_u", [8_954]), 0);

// memory_copy.wast:2205
assert_return(() => call($14, "load8_u", [9_153]), 0);

// memory_copy.wast:2206
assert_return(() => call($14, "load8_u", [9_352]), 0);

// memory_copy.wast:2207
assert_return(() => call($14, "load8_u", [9_551]), 0);

// memory_copy.wast:2208
assert_return(() => call($14, "load8_u", [9_750]), 0);

// memory_copy.wast:2209
assert_return(() => call($14, "load8_u", [9_949]), 0);

// memory_copy.wast:2210
assert_return(() => call($14, "load8_u", [10_148]), 0);

// memory_copy.wast:2211
assert_return(() => call($14, "load8_u", [10_347]), 0);

// memory_copy.wast:2212
assert_return(() => call($14, "load8_u", [10_546]), 0);

// memory_copy.wast:2213
assert_return(() => call($14, "load8_u", [10_745]), 0);

// memory_copy.wast:2214
assert_return(() => call($14, "load8_u", [10_944]), 0);

// memory_copy.wast:2215
assert_return(() => call($14, "load8_u", [11_143]), 0);

// memory_copy.wast:2216
assert_return(() => call($14, "load8_u", [11_342]), 0);

// memory_copy.wast:2217
assert_return(() => call($14, "load8_u", [11_541]), 0);

// memory_copy.wast:2218
assert_return(() => call($14, "load8_u", [11_740]), 0);

// memory_copy.wast:2219
assert_return(() => call($14, "load8_u", [11_939]), 0);

// memory_copy.wast:2220
assert_return(() => call($14, "load8_u", [12_138]), 0);

// memory_copy.wast:2221
assert_return(() => call($14, "load8_u", [12_337]), 0);

// memory_copy.wast:2222
assert_return(() => call($14, "load8_u", [12_536]), 0);

// memory_copy.wast:2223
assert_return(() => call($14, "load8_u", [12_735]), 0);

// memory_copy.wast:2224
assert_return(() => call($14, "load8_u", [12_934]), 0);

// memory_copy.wast:2225
assert_return(() => call($14, "load8_u", [13_133]), 0);

// memory_copy.wast:2226
assert_return(() => call($14, "load8_u", [13_332]), 0);

// memory_copy.wast:2227
assert_return(() => call($14, "load8_u", [13_531]), 0);

// memory_copy.wast:2228
assert_return(() => call($14, "load8_u", [13_730]), 0);

// memory_copy.wast:2229
assert_return(() => call($14, "load8_u", [13_929]), 0);

// memory_copy.wast:2230
assert_return(() => call($14, "load8_u", [14_128]), 0);

// memory_copy.wast:2231
assert_return(() => call($14, "load8_u", [14_327]), 0);

// memory_copy.wast:2232
assert_return(() => call($14, "load8_u", [14_526]), 0);

// memory_copy.wast:2233
assert_return(() => call($14, "load8_u", [14_725]), 0);

// memory_copy.wast:2234
assert_return(() => call($14, "load8_u", [14_924]), 0);

// memory_copy.wast:2235
assert_return(() => call($14, "load8_u", [15_123]), 0);

// memory_copy.wast:2236
assert_return(() => call($14, "load8_u", [15_322]), 0);

// memory_copy.wast:2237
assert_return(() => call($14, "load8_u", [15_521]), 0);

// memory_copy.wast:2238
assert_return(() => call($14, "load8_u", [15_720]), 0);

// memory_copy.wast:2239
assert_return(() => call($14, "load8_u", [15_919]), 0);

// memory_copy.wast:2240
assert_return(() => call($14, "load8_u", [16_118]), 0);

// memory_copy.wast:2241
assert_return(() => call($14, "load8_u", [16_317]), 0);

// memory_copy.wast:2242
assert_return(() => call($14, "load8_u", [16_516]), 0);

// memory_copy.wast:2243
assert_return(() => call($14, "load8_u", [16_715]), 0);

// memory_copy.wast:2244
assert_return(() => call($14, "load8_u", [16_914]), 0);

// memory_copy.wast:2245
assert_return(() => call($14, "load8_u", [17_113]), 0);

// memory_copy.wast:2246
assert_return(() => call($14, "load8_u", [17_312]), 0);

// memory_copy.wast:2247
assert_return(() => call($14, "load8_u", [17_511]), 0);

// memory_copy.wast:2248
assert_return(() => call($14, "load8_u", [17_710]), 0);

// memory_copy.wast:2249
assert_return(() => call($14, "load8_u", [17_909]), 0);

// memory_copy.wast:2250
assert_return(() => call($14, "load8_u", [18_108]), 0);

// memory_copy.wast:2251
assert_return(() => call($14, "load8_u", [18_307]), 0);

// memory_copy.wast:2252
assert_return(() => call($14, "load8_u", [18_506]), 0);

// memory_copy.wast:2253
assert_return(() => call($14, "load8_u", [18_705]), 0);

// memory_copy.wast:2254
assert_return(() => call($14, "load8_u", [18_904]), 0);

// memory_copy.wast:2255
assert_return(() => call($14, "load8_u", [19_103]), 0);

// memory_copy.wast:2256
assert_return(() => call($14, "load8_u", [19_302]), 0);

// memory_copy.wast:2257
assert_return(() => call($14, "load8_u", [19_501]), 0);

// memory_copy.wast:2258
assert_return(() => call($14, "load8_u", [19_700]), 0);

// memory_copy.wast:2259
assert_return(() => call($14, "load8_u", [19_899]), 0);

// memory_copy.wast:2260
assert_return(() => call($14, "load8_u", [20_098]), 0);

// memory_copy.wast:2261
assert_return(() => call($14, "load8_u", [20_297]), 0);

// memory_copy.wast:2262
assert_return(() => call($14, "load8_u", [20_496]), 0);

// memory_copy.wast:2263
assert_return(() => call($14, "load8_u", [20_695]), 0);

// memory_copy.wast:2264
assert_return(() => call($14, "load8_u", [20_894]), 0);

// memory_copy.wast:2265
assert_return(() => call($14, "load8_u", [21_093]), 0);

// memory_copy.wast:2266
assert_return(() => call($14, "load8_u", [21_292]), 0);

// memory_copy.wast:2267
assert_return(() => call($14, "load8_u", [21_491]), 0);

// memory_copy.wast:2268
assert_return(() => call($14, "load8_u", [21_690]), 0);

// memory_copy.wast:2269
assert_return(() => call($14, "load8_u", [21_889]), 0);

// memory_copy.wast:2270
assert_return(() => call($14, "load8_u", [22_088]), 0);

// memory_copy.wast:2271
assert_return(() => call($14, "load8_u", [22_287]), 0);

// memory_copy.wast:2272
assert_return(() => call($14, "load8_u", [22_486]), 0);

// memory_copy.wast:2273
assert_return(() => call($14, "load8_u", [22_685]), 0);

// memory_copy.wast:2274
assert_return(() => call($14, "load8_u", [22_884]), 0);

// memory_copy.wast:2275
assert_return(() => call($14, "load8_u", [23_083]), 0);

// memory_copy.wast:2276
assert_return(() => call($14, "load8_u", [23_282]), 0);

// memory_copy.wast:2277
assert_return(() => call($14, "load8_u", [23_481]), 0);

// memory_copy.wast:2278
assert_return(() => call($14, "load8_u", [23_680]), 0);

// memory_copy.wast:2279
assert_return(() => call($14, "load8_u", [23_879]), 0);

// memory_copy.wast:2280
assert_return(() => call($14, "load8_u", [24_078]), 0);

// memory_copy.wast:2281
assert_return(() => call($14, "load8_u", [24_277]), 0);

// memory_copy.wast:2282
assert_return(() => call($14, "load8_u", [24_476]), 0);

// memory_copy.wast:2283
assert_return(() => call($14, "load8_u", [24_675]), 0);

// memory_copy.wast:2284
assert_return(() => call($14, "load8_u", [24_874]), 0);

// memory_copy.wast:2285
assert_return(() => call($14, "load8_u", [25_073]), 0);

// memory_copy.wast:2286
assert_return(() => call($14, "load8_u", [25_272]), 0);

// memory_copy.wast:2287
assert_return(() => call($14, "load8_u", [25_471]), 0);

// memory_copy.wast:2288
assert_return(() => call($14, "load8_u", [25_670]), 0);

// memory_copy.wast:2289
assert_return(() => call($14, "load8_u", [25_869]), 0);

// memory_copy.wast:2290
assert_return(() => call($14, "load8_u", [26_068]), 0);

// memory_copy.wast:2291
assert_return(() => call($14, "load8_u", [26_267]), 0);

// memory_copy.wast:2292
assert_return(() => call($14, "load8_u", [26_466]), 0);

// memory_copy.wast:2293
assert_return(() => call($14, "load8_u", [26_665]), 0);

// memory_copy.wast:2294
assert_return(() => call($14, "load8_u", [26_864]), 0);

// memory_copy.wast:2295
assert_return(() => call($14, "load8_u", [27_063]), 0);

// memory_copy.wast:2296
assert_return(() => call($14, "load8_u", [27_262]), 0);

// memory_copy.wast:2297
assert_return(() => call($14, "load8_u", [27_461]), 0);

// memory_copy.wast:2298
assert_return(() => call($14, "load8_u", [27_660]), 0);

// memory_copy.wast:2299
assert_return(() => call($14, "load8_u", [27_859]), 0);

// memory_copy.wast:2300
assert_return(() => call($14, "load8_u", [28_058]), 0);

// memory_copy.wast:2301
assert_return(() => call($14, "load8_u", [28_257]), 0);

// memory_copy.wast:2302
assert_return(() => call($14, "load8_u", [28_456]), 0);

// memory_copy.wast:2303
assert_return(() => call($14, "load8_u", [28_655]), 0);

// memory_copy.wast:2304
assert_return(() => call($14, "load8_u", [28_854]), 0);

// memory_copy.wast:2305
assert_return(() => call($14, "load8_u", [29_053]), 0);

// memory_copy.wast:2306
assert_return(() => call($14, "load8_u", [29_252]), 0);

// memory_copy.wast:2307
assert_return(() => call($14, "load8_u", [29_451]), 0);

// memory_copy.wast:2308
assert_return(() => call($14, "load8_u", [29_650]), 0);

// memory_copy.wast:2309
assert_return(() => call($14, "load8_u", [29_849]), 0);

// memory_copy.wast:2310
assert_return(() => call($14, "load8_u", [30_048]), 0);

// memory_copy.wast:2311
assert_return(() => call($14, "load8_u", [30_247]), 0);

// memory_copy.wast:2312
assert_return(() => call($14, "load8_u", [30_446]), 0);

// memory_copy.wast:2313
assert_return(() => call($14, "load8_u", [30_645]), 0);

// memory_copy.wast:2314
assert_return(() => call($14, "load8_u", [30_844]), 0);

// memory_copy.wast:2315
assert_return(() => call($14, "load8_u", [31_043]), 0);

// memory_copy.wast:2316
assert_return(() => call($14, "load8_u", [31_242]), 0);

// memory_copy.wast:2317
assert_return(() => call($14, "load8_u", [31_441]), 0);

// memory_copy.wast:2318
assert_return(() => call($14, "load8_u", [31_640]), 0);

// memory_copy.wast:2319
assert_return(() => call($14, "load8_u", [31_839]), 0);

// memory_copy.wast:2320
assert_return(() => call($14, "load8_u", [32_038]), 0);

// memory_copy.wast:2321
assert_return(() => call($14, "load8_u", [32_237]), 0);

// memory_copy.wast:2322
assert_return(() => call($14, "load8_u", [32_436]), 0);

// memory_copy.wast:2323
assert_return(() => call($14, "load8_u", [32_635]), 0);

// memory_copy.wast:2324
assert_return(() => call($14, "load8_u", [32_834]), 0);

// memory_copy.wast:2325
assert_return(() => call($14, "load8_u", [33_033]), 0);

// memory_copy.wast:2326
assert_return(() => call($14, "load8_u", [33_232]), 0);

// memory_copy.wast:2327
assert_return(() => call($14, "load8_u", [33_431]), 0);

// memory_copy.wast:2328
assert_return(() => call($14, "load8_u", [33_630]), 0);

// memory_copy.wast:2329
assert_return(() => call($14, "load8_u", [33_829]), 0);

// memory_copy.wast:2330
assert_return(() => call($14, "load8_u", [34_028]), 0);

// memory_copy.wast:2331
assert_return(() => call($14, "load8_u", [34_227]), 0);

// memory_copy.wast:2332
assert_return(() => call($14, "load8_u", [34_426]), 0);

// memory_copy.wast:2333
assert_return(() => call($14, "load8_u", [34_625]), 0);

// memory_copy.wast:2334
assert_return(() => call($14, "load8_u", [34_824]), 0);

// memory_copy.wast:2335
assert_return(() => call($14, "load8_u", [35_023]), 0);

// memory_copy.wast:2336
assert_return(() => call($14, "load8_u", [35_222]), 0);

// memory_copy.wast:2337
assert_return(() => call($14, "load8_u", [35_421]), 0);

// memory_copy.wast:2338
assert_return(() => call($14, "load8_u", [35_620]), 0);

// memory_copy.wast:2339
assert_return(() => call($14, "load8_u", [35_819]), 0);

// memory_copy.wast:2340
assert_return(() => call($14, "load8_u", [36_018]), 0);

// memory_copy.wast:2341
assert_return(() => call($14, "load8_u", [36_217]), 0);

// memory_copy.wast:2342
assert_return(() => call($14, "load8_u", [36_416]), 0);

// memory_copy.wast:2343
assert_return(() => call($14, "load8_u", [36_615]), 0);

// memory_copy.wast:2344
assert_return(() => call($14, "load8_u", [36_814]), 0);

// memory_copy.wast:2345
assert_return(() => call($14, "load8_u", [37_013]), 0);

// memory_copy.wast:2346
assert_return(() => call($14, "load8_u", [37_212]), 0);

// memory_copy.wast:2347
assert_return(() => call($14, "load8_u", [37_411]), 0);

// memory_copy.wast:2348
assert_return(() => call($14, "load8_u", [37_610]), 0);

// memory_copy.wast:2349
assert_return(() => call($14, "load8_u", [37_809]), 0);

// memory_copy.wast:2350
assert_return(() => call($14, "load8_u", [38_008]), 0);

// memory_copy.wast:2351
assert_return(() => call($14, "load8_u", [38_207]), 0);

// memory_copy.wast:2352
assert_return(() => call($14, "load8_u", [38_406]), 0);

// memory_copy.wast:2353
assert_return(() => call($14, "load8_u", [38_605]), 0);

// memory_copy.wast:2354
assert_return(() => call($14, "load8_u", [38_804]), 0);

// memory_copy.wast:2355
assert_return(() => call($14, "load8_u", [39_003]), 0);

// memory_copy.wast:2356
assert_return(() => call($14, "load8_u", [39_202]), 0);

// memory_copy.wast:2357
assert_return(() => call($14, "load8_u", [39_401]), 0);

// memory_copy.wast:2358
assert_return(() => call($14, "load8_u", [39_600]), 0);

// memory_copy.wast:2359
assert_return(() => call($14, "load8_u", [39_799]), 0);

// memory_copy.wast:2360
assert_return(() => call($14, "load8_u", [39_998]), 0);

// memory_copy.wast:2361
assert_return(() => call($14, "load8_u", [40_197]), 0);

// memory_copy.wast:2362
assert_return(() => call($14, "load8_u", [40_396]), 0);

// memory_copy.wast:2363
assert_return(() => call($14, "load8_u", [40_595]), 0);

// memory_copy.wast:2364
assert_return(() => call($14, "load8_u", [40_794]), 0);

// memory_copy.wast:2365
assert_return(() => call($14, "load8_u", [40_993]), 0);

// memory_copy.wast:2366
assert_return(() => call($14, "load8_u", [41_192]), 0);

// memory_copy.wast:2367
assert_return(() => call($14, "load8_u", [41_391]), 0);

// memory_copy.wast:2368
assert_return(() => call($14, "load8_u", [41_590]), 0);

// memory_copy.wast:2369
assert_return(() => call($14, "load8_u", [41_789]), 0);

// memory_copy.wast:2370
assert_return(() => call($14, "load8_u", [41_988]), 0);

// memory_copy.wast:2371
assert_return(() => call($14, "load8_u", [42_187]), 0);

// memory_copy.wast:2372
assert_return(() => call($14, "load8_u", [42_386]), 0);

// memory_copy.wast:2373
assert_return(() => call($14, "load8_u", [42_585]), 0);

// memory_copy.wast:2374
assert_return(() => call($14, "load8_u", [42_784]), 0);

// memory_copy.wast:2375
assert_return(() => call($14, "load8_u", [42_983]), 0);

// memory_copy.wast:2376
assert_return(() => call($14, "load8_u", [43_182]), 0);

// memory_copy.wast:2377
assert_return(() => call($14, "load8_u", [43_381]), 0);

// memory_copy.wast:2378
assert_return(() => call($14, "load8_u", [43_580]), 0);

// memory_copy.wast:2379
assert_return(() => call($14, "load8_u", [43_779]), 0);

// memory_copy.wast:2380
assert_return(() => call($14, "load8_u", [43_978]), 0);

// memory_copy.wast:2381
assert_return(() => call($14, "load8_u", [44_177]), 0);

// memory_copy.wast:2382
assert_return(() => call($14, "load8_u", [44_376]), 0);

// memory_copy.wast:2383
assert_return(() => call($14, "load8_u", [44_575]), 0);

// memory_copy.wast:2384
assert_return(() => call($14, "load8_u", [44_774]), 0);

// memory_copy.wast:2385
assert_return(() => call($14, "load8_u", [44_973]), 0);

// memory_copy.wast:2386
assert_return(() => call($14, "load8_u", [45_172]), 0);

// memory_copy.wast:2387
assert_return(() => call($14, "load8_u", [45_371]), 0);

// memory_copy.wast:2388
assert_return(() => call($14, "load8_u", [45_570]), 0);

// memory_copy.wast:2389
assert_return(() => call($14, "load8_u", [45_769]), 0);

// memory_copy.wast:2390
assert_return(() => call($14, "load8_u", [45_968]), 0);

// memory_copy.wast:2391
assert_return(() => call($14, "load8_u", [46_167]), 0);

// memory_copy.wast:2392
assert_return(() => call($14, "load8_u", [46_366]), 0);

// memory_copy.wast:2393
assert_return(() => call($14, "load8_u", [46_565]), 0);

// memory_copy.wast:2394
assert_return(() => call($14, "load8_u", [46_764]), 0);

// memory_copy.wast:2395
assert_return(() => call($14, "load8_u", [46_963]), 0);

// memory_copy.wast:2396
assert_return(() => call($14, "load8_u", [47_162]), 0);

// memory_copy.wast:2397
assert_return(() => call($14, "load8_u", [47_361]), 0);

// memory_copy.wast:2398
assert_return(() => call($14, "load8_u", [47_560]), 0);

// memory_copy.wast:2399
assert_return(() => call($14, "load8_u", [47_759]), 0);

// memory_copy.wast:2400
assert_return(() => call($14, "load8_u", [47_958]), 0);

// memory_copy.wast:2401
assert_return(() => call($14, "load8_u", [48_157]), 0);

// memory_copy.wast:2402
assert_return(() => call($14, "load8_u", [48_356]), 0);

// memory_copy.wast:2403
assert_return(() => call($14, "load8_u", [48_555]), 0);

// memory_copy.wast:2404
assert_return(() => call($14, "load8_u", [48_754]), 0);

// memory_copy.wast:2405
assert_return(() => call($14, "load8_u", [48_953]), 0);

// memory_copy.wast:2406
assert_return(() => call($14, "load8_u", [49_152]), 0);

// memory_copy.wast:2407
assert_return(() => call($14, "load8_u", [49_351]), 0);

// memory_copy.wast:2408
assert_return(() => call($14, "load8_u", [49_550]), 0);

// memory_copy.wast:2409
assert_return(() => call($14, "load8_u", [49_749]), 0);

// memory_copy.wast:2410
assert_return(() => call($14, "load8_u", [49_948]), 0);

// memory_copy.wast:2411
assert_return(() => call($14, "load8_u", [50_147]), 0);

// memory_copy.wast:2412
assert_return(() => call($14, "load8_u", [50_346]), 0);

// memory_copy.wast:2413
assert_return(() => call($14, "load8_u", [50_545]), 0);

// memory_copy.wast:2414
assert_return(() => call($14, "load8_u", [50_744]), 0);

// memory_copy.wast:2415
assert_return(() => call($14, "load8_u", [50_943]), 0);

// memory_copy.wast:2416
assert_return(() => call($14, "load8_u", [51_142]), 0);

// memory_copy.wast:2417
assert_return(() => call($14, "load8_u", [51_341]), 0);

// memory_copy.wast:2418
assert_return(() => call($14, "load8_u", [51_540]), 0);

// memory_copy.wast:2419
assert_return(() => call($14, "load8_u", [51_739]), 0);

// memory_copy.wast:2420
assert_return(() => call($14, "load8_u", [51_938]), 0);

// memory_copy.wast:2421
assert_return(() => call($14, "load8_u", [52_137]), 0);

// memory_copy.wast:2422
assert_return(() => call($14, "load8_u", [52_336]), 0);

// memory_copy.wast:2423
assert_return(() => call($14, "load8_u", [52_535]), 0);

// memory_copy.wast:2424
assert_return(() => call($14, "load8_u", [52_734]), 0);

// memory_copy.wast:2425
assert_return(() => call($14, "load8_u", [52_933]), 0);

// memory_copy.wast:2426
assert_return(() => call($14, "load8_u", [53_132]), 0);

// memory_copy.wast:2427
assert_return(() => call($14, "load8_u", [53_331]), 0);

// memory_copy.wast:2428
assert_return(() => call($14, "load8_u", [53_530]), 0);

// memory_copy.wast:2429
assert_return(() => call($14, "load8_u", [53_729]), 0);

// memory_copy.wast:2430
assert_return(() => call($14, "load8_u", [53_928]), 0);

// memory_copy.wast:2431
assert_return(() => call($14, "load8_u", [54_127]), 0);

// memory_copy.wast:2432
assert_return(() => call($14, "load8_u", [54_326]), 0);

// memory_copy.wast:2433
assert_return(() => call($14, "load8_u", [54_525]), 0);

// memory_copy.wast:2434
assert_return(() => call($14, "load8_u", [54_724]), 0);

// memory_copy.wast:2435
assert_return(() => call($14, "load8_u", [54_923]), 0);

// memory_copy.wast:2436
assert_return(() => call($14, "load8_u", [55_122]), 0);

// memory_copy.wast:2437
assert_return(() => call($14, "load8_u", [55_321]), 0);

// memory_copy.wast:2438
assert_return(() => call($14, "load8_u", [55_520]), 0);

// memory_copy.wast:2439
assert_return(() => call($14, "load8_u", [55_719]), 0);

// memory_copy.wast:2440
assert_return(() => call($14, "load8_u", [55_918]), 0);

// memory_copy.wast:2441
assert_return(() => call($14, "load8_u", [56_117]), 0);

// memory_copy.wast:2442
assert_return(() => call($14, "load8_u", [56_316]), 0);

// memory_copy.wast:2443
assert_return(() => call($14, "load8_u", [56_515]), 0);

// memory_copy.wast:2444
assert_return(() => call($14, "load8_u", [56_714]), 0);

// memory_copy.wast:2445
assert_return(() => call($14, "load8_u", [56_913]), 0);

// memory_copy.wast:2446
assert_return(() => call($14, "load8_u", [57_112]), 0);

// memory_copy.wast:2447
assert_return(() => call($14, "load8_u", [57_311]), 0);

// memory_copy.wast:2448
assert_return(() => call($14, "load8_u", [57_510]), 0);

// memory_copy.wast:2449
assert_return(() => call($14, "load8_u", [57_709]), 0);

// memory_copy.wast:2450
assert_return(() => call($14, "load8_u", [57_908]), 0);

// memory_copy.wast:2451
assert_return(() => call($14, "load8_u", [58_107]), 0);

// memory_copy.wast:2452
assert_return(() => call($14, "load8_u", [58_306]), 0);

// memory_copy.wast:2453
assert_return(() => call($14, "load8_u", [58_505]), 0);

// memory_copy.wast:2454
assert_return(() => call($14, "load8_u", [58_704]), 0);

// memory_copy.wast:2455
assert_return(() => call($14, "load8_u", [58_903]), 0);

// memory_copy.wast:2456
assert_return(() => call($14, "load8_u", [59_102]), 0);

// memory_copy.wast:2457
assert_return(() => call($14, "load8_u", [59_301]), 0);

// memory_copy.wast:2458
assert_return(() => call($14, "load8_u", [59_500]), 0);

// memory_copy.wast:2459
assert_return(() => call($14, "load8_u", [59_699]), 0);

// memory_copy.wast:2460
assert_return(() => call($14, "load8_u", [59_898]), 0);

// memory_copy.wast:2461
assert_return(() => call($14, "load8_u", [60_097]), 0);

// memory_copy.wast:2462
assert_return(() => call($14, "load8_u", [60_296]), 0);

// memory_copy.wast:2463
assert_return(() => call($14, "load8_u", [60_495]), 0);

// memory_copy.wast:2464
assert_return(() => call($14, "load8_u", [60_694]), 0);

// memory_copy.wast:2465
assert_return(() => call($14, "load8_u", [60_893]), 0);

// memory_copy.wast:2466
assert_return(() => call($14, "load8_u", [61_092]), 0);

// memory_copy.wast:2467
assert_return(() => call($14, "load8_u", [61_291]), 0);

// memory_copy.wast:2468
assert_return(() => call($14, "load8_u", [61_490]), 0);

// memory_copy.wast:2469
assert_return(() => call($14, "load8_u", [61_689]), 0);

// memory_copy.wast:2470
assert_return(() => call($14, "load8_u", [61_888]), 0);

// memory_copy.wast:2471
assert_return(() => call($14, "load8_u", [62_087]), 0);

// memory_copy.wast:2472
assert_return(() => call($14, "load8_u", [62_286]), 0);

// memory_copy.wast:2473
assert_return(() => call($14, "load8_u", [62_485]), 0);

// memory_copy.wast:2474
assert_return(() => call($14, "load8_u", [62_684]), 0);

// memory_copy.wast:2475
assert_return(() => call($14, "load8_u", [62_883]), 0);

// memory_copy.wast:2476
assert_return(() => call($14, "load8_u", [63_082]), 0);

// memory_copy.wast:2477
assert_return(() => call($14, "load8_u", [63_281]), 0);

// memory_copy.wast:2478
assert_return(() => call($14, "load8_u", [63_480]), 0);

// memory_copy.wast:2479
assert_return(() => call($14, "load8_u", [63_679]), 0);

// memory_copy.wast:2480
assert_return(() => call($14, "load8_u", [63_878]), 0);

// memory_copy.wast:2481
assert_return(() => call($14, "load8_u", [64_077]), 0);

// memory_copy.wast:2482
assert_return(() => call($14, "load8_u", [64_276]), 0);

// memory_copy.wast:2483
assert_return(() => call($14, "load8_u", [64_475]), 0);

// memory_copy.wast:2484
assert_return(() => call($14, "load8_u", [64_674]), 0);

// memory_copy.wast:2485
assert_return(() => call($14, "load8_u", [64_873]), 0);

// memory_copy.wast:2486
assert_return(() => call($14, "load8_u", [65_072]), 0);

// memory_copy.wast:2487
assert_return(() => call($14, "load8_u", [65_271]), 0);

// memory_copy.wast:2488
assert_return(() => call($14, "load8_u", [65_470]), 0);

// memory_copy.wast:2489
assert_return(() => call($14, "load8_u", [65_516]), 0);

// memory_copy.wast:2490
assert_return(() => call($14, "load8_u", [65_517]), 1);

// memory_copy.wast:2491
assert_return(() => call($14, "load8_u", [65_518]), 2);

// memory_copy.wast:2492
assert_return(() => call($14, "load8_u", [65_519]), 3);

// memory_copy.wast:2493
assert_return(() => call($14, "load8_u", [65_520]), 4);

// memory_copy.wast:2494
assert_return(() => call($14, "load8_u", [65_521]), 5);

// memory_copy.wast:2495
assert_return(() => call($14, "load8_u", [65_522]), 6);

// memory_copy.wast:2496
assert_return(() => call($14, "load8_u", [65_523]), 7);

// memory_copy.wast:2497
assert_return(() => call($14, "load8_u", [65_524]), 8);

// memory_copy.wast:2498
assert_return(() => call($14, "load8_u", [65_525]), 9);

// memory_copy.wast:2499
assert_return(() => call($14, "load8_u", [65_526]), 10);

// memory_copy.wast:2500
assert_return(() => call($14, "load8_u", [65_527]), 11);

// memory_copy.wast:2501
assert_return(() => call($14, "load8_u", [65_528]), 12);

// memory_copy.wast:2502
assert_return(() => call($14, "load8_u", [65_529]), 13);

// memory_copy.wast:2503
assert_return(() => call($14, "load8_u", [65_530]), 14);

// memory_copy.wast:2504
assert_return(() => call($14, "load8_u", [65_531]), 15);

// memory_copy.wast:2505
assert_return(() => call($14, "load8_u", [65_532]), 16);

// memory_copy.wast:2506
assert_return(() => call($14, "load8_u", [65_533]), 17);

// memory_copy.wast:2507
assert_return(() => call($14, "load8_u", [65_534]), 18);

// memory_copy.wast:2508
assert_return(() => call($14, "load8_u", [65_535]), 19);

// memory_copy.wast:2510
let $15 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9c\x80\x80\x80\x00\x01\x00\x41\xe2\xff\x03\x0b\x14\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

// memory_copy.wast:2518
assert_trap(() => call($15, "run", [65_516, 65_506, 40]));

// memory_copy.wast:2521
assert_return(() => call($15, "load8_u", [198]), 0);

// memory_copy.wast:2522
assert_return(() => call($15, "load8_u", [397]), 0);

// memory_copy.wast:2523
assert_return(() => call($15, "load8_u", [596]), 0);

// memory_copy.wast:2524
assert_return(() => call($15, "load8_u", [795]), 0);

// memory_copy.wast:2525
assert_return(() => call($15, "load8_u", [994]), 0);

// memory_copy.wast:2526
assert_return(() => call($15, "load8_u", [1_193]), 0);

// memory_copy.wast:2527
assert_return(() => call($15, "load8_u", [1_392]), 0);

// memory_copy.wast:2528
assert_return(() => call($15, "load8_u", [1_591]), 0);

// memory_copy.wast:2529
assert_return(() => call($15, "load8_u", [1_790]), 0);

// memory_copy.wast:2530
assert_return(() => call($15, "load8_u", [1_989]), 0);

// memory_copy.wast:2531
assert_return(() => call($15, "load8_u", [2_188]), 0);

// memory_copy.wast:2532
assert_return(() => call($15, "load8_u", [2_387]), 0);

// memory_copy.wast:2533
assert_return(() => call($15, "load8_u", [2_586]), 0);

// memory_copy.wast:2534
assert_return(() => call($15, "load8_u", [2_785]), 0);

// memory_copy.wast:2535
assert_return(() => call($15, "load8_u", [2_984]), 0);

// memory_copy.wast:2536
assert_return(() => call($15, "load8_u", [3_183]), 0);

// memory_copy.wast:2537
assert_return(() => call($15, "load8_u", [3_382]), 0);

// memory_copy.wast:2538
assert_return(() => call($15, "load8_u", [3_581]), 0);

// memory_copy.wast:2539
assert_return(() => call($15, "load8_u", [3_780]), 0);

// memory_copy.wast:2540
assert_return(() => call($15, "load8_u", [3_979]), 0);

// memory_copy.wast:2541
assert_return(() => call($15, "load8_u", [4_178]), 0);

// memory_copy.wast:2542
assert_return(() => call($15, "load8_u", [4_377]), 0);

// memory_copy.wast:2543
assert_return(() => call($15, "load8_u", [4_576]), 0);

// memory_copy.wast:2544
assert_return(() => call($15, "load8_u", [4_775]), 0);

// memory_copy.wast:2545
assert_return(() => call($15, "load8_u", [4_974]), 0);

// memory_copy.wast:2546
assert_return(() => call($15, "load8_u", [5_173]), 0);

// memory_copy.wast:2547
assert_return(() => call($15, "load8_u", [5_372]), 0);

// memory_copy.wast:2548
assert_return(() => call($15, "load8_u", [5_571]), 0);

// memory_copy.wast:2549
assert_return(() => call($15, "load8_u", [5_770]), 0);

// memory_copy.wast:2550
assert_return(() => call($15, "load8_u", [5_969]), 0);

// memory_copy.wast:2551
assert_return(() => call($15, "load8_u", [6_168]), 0);

// memory_copy.wast:2552
assert_return(() => call($15, "load8_u", [6_367]), 0);

// memory_copy.wast:2553
assert_return(() => call($15, "load8_u", [6_566]), 0);

// memory_copy.wast:2554
assert_return(() => call($15, "load8_u", [6_765]), 0);

// memory_copy.wast:2555
assert_return(() => call($15, "load8_u", [6_964]), 0);

// memory_copy.wast:2556
assert_return(() => call($15, "load8_u", [7_163]), 0);

// memory_copy.wast:2557
assert_return(() => call($15, "load8_u", [7_362]), 0);

// memory_copy.wast:2558
assert_return(() => call($15, "load8_u", [7_561]), 0);

// memory_copy.wast:2559
assert_return(() => call($15, "load8_u", [7_760]), 0);

// memory_copy.wast:2560
assert_return(() => call($15, "load8_u", [7_959]), 0);

// memory_copy.wast:2561
assert_return(() => call($15, "load8_u", [8_158]), 0);

// memory_copy.wast:2562
assert_return(() => call($15, "load8_u", [8_357]), 0);

// memory_copy.wast:2563
assert_return(() => call($15, "load8_u", [8_556]), 0);

// memory_copy.wast:2564
assert_return(() => call($15, "load8_u", [8_755]), 0);

// memory_copy.wast:2565
assert_return(() => call($15, "load8_u", [8_954]), 0);

// memory_copy.wast:2566
assert_return(() => call($15, "load8_u", [9_153]), 0);

// memory_copy.wast:2567
assert_return(() => call($15, "load8_u", [9_352]), 0);

// memory_copy.wast:2568
assert_return(() => call($15, "load8_u", [9_551]), 0);

// memory_copy.wast:2569
assert_return(() => call($15, "load8_u", [9_750]), 0);

// memory_copy.wast:2570
assert_return(() => call($15, "load8_u", [9_949]), 0);

// memory_copy.wast:2571
assert_return(() => call($15, "load8_u", [10_148]), 0);

// memory_copy.wast:2572
assert_return(() => call($15, "load8_u", [10_347]), 0);

// memory_copy.wast:2573
assert_return(() => call($15, "load8_u", [10_546]), 0);

// memory_copy.wast:2574
assert_return(() => call($15, "load8_u", [10_745]), 0);

// memory_copy.wast:2575
assert_return(() => call($15, "load8_u", [10_944]), 0);

// memory_copy.wast:2576
assert_return(() => call($15, "load8_u", [11_143]), 0);

// memory_copy.wast:2577
assert_return(() => call($15, "load8_u", [11_342]), 0);

// memory_copy.wast:2578
assert_return(() => call($15, "load8_u", [11_541]), 0);

// memory_copy.wast:2579
assert_return(() => call($15, "load8_u", [11_740]), 0);

// memory_copy.wast:2580
assert_return(() => call($15, "load8_u", [11_939]), 0);

// memory_copy.wast:2581
assert_return(() => call($15, "load8_u", [12_138]), 0);

// memory_copy.wast:2582
assert_return(() => call($15, "load8_u", [12_337]), 0);

// memory_copy.wast:2583
assert_return(() => call($15, "load8_u", [12_536]), 0);

// memory_copy.wast:2584
assert_return(() => call($15, "load8_u", [12_735]), 0);

// memory_copy.wast:2585
assert_return(() => call($15, "load8_u", [12_934]), 0);

// memory_copy.wast:2586
assert_return(() => call($15, "load8_u", [13_133]), 0);

// memory_copy.wast:2587
assert_return(() => call($15, "load8_u", [13_332]), 0);

// memory_copy.wast:2588
assert_return(() => call($15, "load8_u", [13_531]), 0);

// memory_copy.wast:2589
assert_return(() => call($15, "load8_u", [13_730]), 0);

// memory_copy.wast:2590
assert_return(() => call($15, "load8_u", [13_929]), 0);

// memory_copy.wast:2591
assert_return(() => call($15, "load8_u", [14_128]), 0);

// memory_copy.wast:2592
assert_return(() => call($15, "load8_u", [14_327]), 0);

// memory_copy.wast:2593
assert_return(() => call($15, "load8_u", [14_526]), 0);

// memory_copy.wast:2594
assert_return(() => call($15, "load8_u", [14_725]), 0);

// memory_copy.wast:2595
assert_return(() => call($15, "load8_u", [14_924]), 0);

// memory_copy.wast:2596
assert_return(() => call($15, "load8_u", [15_123]), 0);

// memory_copy.wast:2597
assert_return(() => call($15, "load8_u", [15_322]), 0);

// memory_copy.wast:2598
assert_return(() => call($15, "load8_u", [15_521]), 0);

// memory_copy.wast:2599
assert_return(() => call($15, "load8_u", [15_720]), 0);

// memory_copy.wast:2600
assert_return(() => call($15, "load8_u", [15_919]), 0);

// memory_copy.wast:2601
assert_return(() => call($15, "load8_u", [16_118]), 0);

// memory_copy.wast:2602
assert_return(() => call($15, "load8_u", [16_317]), 0);

// memory_copy.wast:2603
assert_return(() => call($15, "load8_u", [16_516]), 0);

// memory_copy.wast:2604
assert_return(() => call($15, "load8_u", [16_715]), 0);

// memory_copy.wast:2605
assert_return(() => call($15, "load8_u", [16_914]), 0);

// memory_copy.wast:2606
assert_return(() => call($15, "load8_u", [17_113]), 0);

// memory_copy.wast:2607
assert_return(() => call($15, "load8_u", [17_312]), 0);

// memory_copy.wast:2608
assert_return(() => call($15, "load8_u", [17_511]), 0);

// memory_copy.wast:2609
assert_return(() => call($15, "load8_u", [17_710]), 0);

// memory_copy.wast:2610
assert_return(() => call($15, "load8_u", [17_909]), 0);

// memory_copy.wast:2611
assert_return(() => call($15, "load8_u", [18_108]), 0);

// memory_copy.wast:2612
assert_return(() => call($15, "load8_u", [18_307]), 0);

// memory_copy.wast:2613
assert_return(() => call($15, "load8_u", [18_506]), 0);

// memory_copy.wast:2614
assert_return(() => call($15, "load8_u", [18_705]), 0);

// memory_copy.wast:2615
assert_return(() => call($15, "load8_u", [18_904]), 0);

// memory_copy.wast:2616
assert_return(() => call($15, "load8_u", [19_103]), 0);

// memory_copy.wast:2617
assert_return(() => call($15, "load8_u", [19_302]), 0);

// memory_copy.wast:2618
assert_return(() => call($15, "load8_u", [19_501]), 0);

// memory_copy.wast:2619
assert_return(() => call($15, "load8_u", [19_700]), 0);

// memory_copy.wast:2620
assert_return(() => call($15, "load8_u", [19_899]), 0);

// memory_copy.wast:2621
assert_return(() => call($15, "load8_u", [20_098]), 0);

// memory_copy.wast:2622
assert_return(() => call($15, "load8_u", [20_297]), 0);

// memory_copy.wast:2623
assert_return(() => call($15, "load8_u", [20_496]), 0);

// memory_copy.wast:2624
assert_return(() => call($15, "load8_u", [20_695]), 0);

// memory_copy.wast:2625
assert_return(() => call($15, "load8_u", [20_894]), 0);

// memory_copy.wast:2626
assert_return(() => call($15, "load8_u", [21_093]), 0);

// memory_copy.wast:2627
assert_return(() => call($15, "load8_u", [21_292]), 0);

// memory_copy.wast:2628
assert_return(() => call($15, "load8_u", [21_491]), 0);

// memory_copy.wast:2629
assert_return(() => call($15, "load8_u", [21_690]), 0);

// memory_copy.wast:2630
assert_return(() => call($15, "load8_u", [21_889]), 0);

// memory_copy.wast:2631
assert_return(() => call($15, "load8_u", [22_088]), 0);

// memory_copy.wast:2632
assert_return(() => call($15, "load8_u", [22_287]), 0);

// memory_copy.wast:2633
assert_return(() => call($15, "load8_u", [22_486]), 0);

// memory_copy.wast:2634
assert_return(() => call($15, "load8_u", [22_685]), 0);

// memory_copy.wast:2635
assert_return(() => call($15, "load8_u", [22_884]), 0);

// memory_copy.wast:2636
assert_return(() => call($15, "load8_u", [23_083]), 0);

// memory_copy.wast:2637
assert_return(() => call($15, "load8_u", [23_282]), 0);

// memory_copy.wast:2638
assert_return(() => call($15, "load8_u", [23_481]), 0);

// memory_copy.wast:2639
assert_return(() => call($15, "load8_u", [23_680]), 0);

// memory_copy.wast:2640
assert_return(() => call($15, "load8_u", [23_879]), 0);

// memory_copy.wast:2641
assert_return(() => call($15, "load8_u", [24_078]), 0);

// memory_copy.wast:2642
assert_return(() => call($15, "load8_u", [24_277]), 0);

// memory_copy.wast:2643
assert_return(() => call($15, "load8_u", [24_476]), 0);

// memory_copy.wast:2644
assert_return(() => call($15, "load8_u", [24_675]), 0);

// memory_copy.wast:2645
assert_return(() => call($15, "load8_u", [24_874]), 0);

// memory_copy.wast:2646
assert_return(() => call($15, "load8_u", [25_073]), 0);

// memory_copy.wast:2647
assert_return(() => call($15, "load8_u", [25_272]), 0);

// memory_copy.wast:2648
assert_return(() => call($15, "load8_u", [25_471]), 0);

// memory_copy.wast:2649
assert_return(() => call($15, "load8_u", [25_670]), 0);

// memory_copy.wast:2650
assert_return(() => call($15, "load8_u", [25_869]), 0);

// memory_copy.wast:2651
assert_return(() => call($15, "load8_u", [26_068]), 0);

// memory_copy.wast:2652
assert_return(() => call($15, "load8_u", [26_267]), 0);

// memory_copy.wast:2653
assert_return(() => call($15, "load8_u", [26_466]), 0);

// memory_copy.wast:2654
assert_return(() => call($15, "load8_u", [26_665]), 0);

// memory_copy.wast:2655
assert_return(() => call($15, "load8_u", [26_864]), 0);

// memory_copy.wast:2656
assert_return(() => call($15, "load8_u", [27_063]), 0);

// memory_copy.wast:2657
assert_return(() => call($15, "load8_u", [27_262]), 0);

// memory_copy.wast:2658
assert_return(() => call($15, "load8_u", [27_461]), 0);

// memory_copy.wast:2659
assert_return(() => call($15, "load8_u", [27_660]), 0);

// memory_copy.wast:2660
assert_return(() => call($15, "load8_u", [27_859]), 0);

// memory_copy.wast:2661
assert_return(() => call($15, "load8_u", [28_058]), 0);

// memory_copy.wast:2662
assert_return(() => call($15, "load8_u", [28_257]), 0);

// memory_copy.wast:2663
assert_return(() => call($15, "load8_u", [28_456]), 0);

// memory_copy.wast:2664
assert_return(() => call($15, "load8_u", [28_655]), 0);

// memory_copy.wast:2665
assert_return(() => call($15, "load8_u", [28_854]), 0);

// memory_copy.wast:2666
assert_return(() => call($15, "load8_u", [29_053]), 0);

// memory_copy.wast:2667
assert_return(() => call($15, "load8_u", [29_252]), 0);

// memory_copy.wast:2668
assert_return(() => call($15, "load8_u", [29_451]), 0);

// memory_copy.wast:2669
assert_return(() => call($15, "load8_u", [29_650]), 0);

// memory_copy.wast:2670
assert_return(() => call($15, "load8_u", [29_849]), 0);

// memory_copy.wast:2671
assert_return(() => call($15, "load8_u", [30_048]), 0);

// memory_copy.wast:2672
assert_return(() => call($15, "load8_u", [30_247]), 0);

// memory_copy.wast:2673
assert_return(() => call($15, "load8_u", [30_446]), 0);

// memory_copy.wast:2674
assert_return(() => call($15, "load8_u", [30_645]), 0);

// memory_copy.wast:2675
assert_return(() => call($15, "load8_u", [30_844]), 0);

// memory_copy.wast:2676
assert_return(() => call($15, "load8_u", [31_043]), 0);

// memory_copy.wast:2677
assert_return(() => call($15, "load8_u", [31_242]), 0);

// memory_copy.wast:2678
assert_return(() => call($15, "load8_u", [31_441]), 0);

// memory_copy.wast:2679
assert_return(() => call($15, "load8_u", [31_640]), 0);

// memory_copy.wast:2680
assert_return(() => call($15, "load8_u", [31_839]), 0);

// memory_copy.wast:2681
assert_return(() => call($15, "load8_u", [32_038]), 0);

// memory_copy.wast:2682
assert_return(() => call($15, "load8_u", [32_237]), 0);

// memory_copy.wast:2683
assert_return(() => call($15, "load8_u", [32_436]), 0);

// memory_copy.wast:2684
assert_return(() => call($15, "load8_u", [32_635]), 0);

// memory_copy.wast:2685
assert_return(() => call($15, "load8_u", [32_834]), 0);

// memory_copy.wast:2686
assert_return(() => call($15, "load8_u", [33_033]), 0);

// memory_copy.wast:2687
assert_return(() => call($15, "load8_u", [33_232]), 0);

// memory_copy.wast:2688
assert_return(() => call($15, "load8_u", [33_431]), 0);

// memory_copy.wast:2689
assert_return(() => call($15, "load8_u", [33_630]), 0);

// memory_copy.wast:2690
assert_return(() => call($15, "load8_u", [33_829]), 0);

// memory_copy.wast:2691
assert_return(() => call($15, "load8_u", [34_028]), 0);

// memory_copy.wast:2692
assert_return(() => call($15, "load8_u", [34_227]), 0);

// memory_copy.wast:2693
assert_return(() => call($15, "load8_u", [34_426]), 0);

// memory_copy.wast:2694
assert_return(() => call($15, "load8_u", [34_625]), 0);

// memory_copy.wast:2695
assert_return(() => call($15, "load8_u", [34_824]), 0);

// memory_copy.wast:2696
assert_return(() => call($15, "load8_u", [35_023]), 0);

// memory_copy.wast:2697
assert_return(() => call($15, "load8_u", [35_222]), 0);

// memory_copy.wast:2698
assert_return(() => call($15, "load8_u", [35_421]), 0);

// memory_copy.wast:2699
assert_return(() => call($15, "load8_u", [35_620]), 0);

// memory_copy.wast:2700
assert_return(() => call($15, "load8_u", [35_819]), 0);

// memory_copy.wast:2701
assert_return(() => call($15, "load8_u", [36_018]), 0);

// memory_copy.wast:2702
assert_return(() => call($15, "load8_u", [36_217]), 0);

// memory_copy.wast:2703
assert_return(() => call($15, "load8_u", [36_416]), 0);

// memory_copy.wast:2704
assert_return(() => call($15, "load8_u", [36_615]), 0);

// memory_copy.wast:2705
assert_return(() => call($15, "load8_u", [36_814]), 0);

// memory_copy.wast:2706
assert_return(() => call($15, "load8_u", [37_013]), 0);

// memory_copy.wast:2707
assert_return(() => call($15, "load8_u", [37_212]), 0);

// memory_copy.wast:2708
assert_return(() => call($15, "load8_u", [37_411]), 0);

// memory_copy.wast:2709
assert_return(() => call($15, "load8_u", [37_610]), 0);

// memory_copy.wast:2710
assert_return(() => call($15, "load8_u", [37_809]), 0);

// memory_copy.wast:2711
assert_return(() => call($15, "load8_u", [38_008]), 0);

// memory_copy.wast:2712
assert_return(() => call($15, "load8_u", [38_207]), 0);

// memory_copy.wast:2713
assert_return(() => call($15, "load8_u", [38_406]), 0);

// memory_copy.wast:2714
assert_return(() => call($15, "load8_u", [38_605]), 0);

// memory_copy.wast:2715
assert_return(() => call($15, "load8_u", [38_804]), 0);

// memory_copy.wast:2716
assert_return(() => call($15, "load8_u", [39_003]), 0);

// memory_copy.wast:2717
assert_return(() => call($15, "load8_u", [39_202]), 0);

// memory_copy.wast:2718
assert_return(() => call($15, "load8_u", [39_401]), 0);

// memory_copy.wast:2719
assert_return(() => call($15, "load8_u", [39_600]), 0);

// memory_copy.wast:2720
assert_return(() => call($15, "load8_u", [39_799]), 0);

// memory_copy.wast:2721
assert_return(() => call($15, "load8_u", [39_998]), 0);

// memory_copy.wast:2722
assert_return(() => call($15, "load8_u", [40_197]), 0);

// memory_copy.wast:2723
assert_return(() => call($15, "load8_u", [40_396]), 0);

// memory_copy.wast:2724
assert_return(() => call($15, "load8_u", [40_595]), 0);

// memory_copy.wast:2725
assert_return(() => call($15, "load8_u", [40_794]), 0);

// memory_copy.wast:2726
assert_return(() => call($15, "load8_u", [40_993]), 0);

// memory_copy.wast:2727
assert_return(() => call($15, "load8_u", [41_192]), 0);

// memory_copy.wast:2728
assert_return(() => call($15, "load8_u", [41_391]), 0);

// memory_copy.wast:2729
assert_return(() => call($15, "load8_u", [41_590]), 0);

// memory_copy.wast:2730
assert_return(() => call($15, "load8_u", [41_789]), 0);

// memory_copy.wast:2731
assert_return(() => call($15, "load8_u", [41_988]), 0);

// memory_copy.wast:2732
assert_return(() => call($15, "load8_u", [42_187]), 0);

// memory_copy.wast:2733
assert_return(() => call($15, "load8_u", [42_386]), 0);

// memory_copy.wast:2734
assert_return(() => call($15, "load8_u", [42_585]), 0);

// memory_copy.wast:2735
assert_return(() => call($15, "load8_u", [42_784]), 0);

// memory_copy.wast:2736
assert_return(() => call($15, "load8_u", [42_983]), 0);

// memory_copy.wast:2737
assert_return(() => call($15, "load8_u", [43_182]), 0);

// memory_copy.wast:2738
assert_return(() => call($15, "load8_u", [43_381]), 0);

// memory_copy.wast:2739
assert_return(() => call($15, "load8_u", [43_580]), 0);

// memory_copy.wast:2740
assert_return(() => call($15, "load8_u", [43_779]), 0);

// memory_copy.wast:2741
assert_return(() => call($15, "load8_u", [43_978]), 0);

// memory_copy.wast:2742
assert_return(() => call($15, "load8_u", [44_177]), 0);

// memory_copy.wast:2743
assert_return(() => call($15, "load8_u", [44_376]), 0);

// memory_copy.wast:2744
assert_return(() => call($15, "load8_u", [44_575]), 0);

// memory_copy.wast:2745
assert_return(() => call($15, "load8_u", [44_774]), 0);

// memory_copy.wast:2746
assert_return(() => call($15, "load8_u", [44_973]), 0);

// memory_copy.wast:2747
assert_return(() => call($15, "load8_u", [45_172]), 0);

// memory_copy.wast:2748
assert_return(() => call($15, "load8_u", [45_371]), 0);

// memory_copy.wast:2749
assert_return(() => call($15, "load8_u", [45_570]), 0);

// memory_copy.wast:2750
assert_return(() => call($15, "load8_u", [45_769]), 0);

// memory_copy.wast:2751
assert_return(() => call($15, "load8_u", [45_968]), 0);

// memory_copy.wast:2752
assert_return(() => call($15, "load8_u", [46_167]), 0);

// memory_copy.wast:2753
assert_return(() => call($15, "load8_u", [46_366]), 0);

// memory_copy.wast:2754
assert_return(() => call($15, "load8_u", [46_565]), 0);

// memory_copy.wast:2755
assert_return(() => call($15, "load8_u", [46_764]), 0);

// memory_copy.wast:2756
assert_return(() => call($15, "load8_u", [46_963]), 0);

// memory_copy.wast:2757
assert_return(() => call($15, "load8_u", [47_162]), 0);

// memory_copy.wast:2758
assert_return(() => call($15, "load8_u", [47_361]), 0);

// memory_copy.wast:2759
assert_return(() => call($15, "load8_u", [47_560]), 0);

// memory_copy.wast:2760
assert_return(() => call($15, "load8_u", [47_759]), 0);

// memory_copy.wast:2761
assert_return(() => call($15, "load8_u", [47_958]), 0);

// memory_copy.wast:2762
assert_return(() => call($15, "load8_u", [48_157]), 0);

// memory_copy.wast:2763
assert_return(() => call($15, "load8_u", [48_356]), 0);

// memory_copy.wast:2764
assert_return(() => call($15, "load8_u", [48_555]), 0);

// memory_copy.wast:2765
assert_return(() => call($15, "load8_u", [48_754]), 0);

// memory_copy.wast:2766
assert_return(() => call($15, "load8_u", [48_953]), 0);

// memory_copy.wast:2767
assert_return(() => call($15, "load8_u", [49_152]), 0);

// memory_copy.wast:2768
assert_return(() => call($15, "load8_u", [49_351]), 0);

// memory_copy.wast:2769
assert_return(() => call($15, "load8_u", [49_550]), 0);

// memory_copy.wast:2770
assert_return(() => call($15, "load8_u", [49_749]), 0);

// memory_copy.wast:2771
assert_return(() => call($15, "load8_u", [49_948]), 0);

// memory_copy.wast:2772
assert_return(() => call($15, "load8_u", [50_147]), 0);

// memory_copy.wast:2773
assert_return(() => call($15, "load8_u", [50_346]), 0);

// memory_copy.wast:2774
assert_return(() => call($15, "load8_u", [50_545]), 0);

// memory_copy.wast:2775
assert_return(() => call($15, "load8_u", [50_744]), 0);

// memory_copy.wast:2776
assert_return(() => call($15, "load8_u", [50_943]), 0);

// memory_copy.wast:2777
assert_return(() => call($15, "load8_u", [51_142]), 0);

// memory_copy.wast:2778
assert_return(() => call($15, "load8_u", [51_341]), 0);

// memory_copy.wast:2779
assert_return(() => call($15, "load8_u", [51_540]), 0);

// memory_copy.wast:2780
assert_return(() => call($15, "load8_u", [51_739]), 0);

// memory_copy.wast:2781
assert_return(() => call($15, "load8_u", [51_938]), 0);

// memory_copy.wast:2782
assert_return(() => call($15, "load8_u", [52_137]), 0);

// memory_copy.wast:2783
assert_return(() => call($15, "load8_u", [52_336]), 0);

// memory_copy.wast:2784
assert_return(() => call($15, "load8_u", [52_535]), 0);

// memory_copy.wast:2785
assert_return(() => call($15, "load8_u", [52_734]), 0);

// memory_copy.wast:2786
assert_return(() => call($15, "load8_u", [52_933]), 0);

// memory_copy.wast:2787
assert_return(() => call($15, "load8_u", [53_132]), 0);

// memory_copy.wast:2788
assert_return(() => call($15, "load8_u", [53_331]), 0);

// memory_copy.wast:2789
assert_return(() => call($15, "load8_u", [53_530]), 0);

// memory_copy.wast:2790
assert_return(() => call($15, "load8_u", [53_729]), 0);

// memory_copy.wast:2791
assert_return(() => call($15, "load8_u", [53_928]), 0);

// memory_copy.wast:2792
assert_return(() => call($15, "load8_u", [54_127]), 0);

// memory_copy.wast:2793
assert_return(() => call($15, "load8_u", [54_326]), 0);

// memory_copy.wast:2794
assert_return(() => call($15, "load8_u", [54_525]), 0);

// memory_copy.wast:2795
assert_return(() => call($15, "load8_u", [54_724]), 0);

// memory_copy.wast:2796
assert_return(() => call($15, "load8_u", [54_923]), 0);

// memory_copy.wast:2797
assert_return(() => call($15, "load8_u", [55_122]), 0);

// memory_copy.wast:2798
assert_return(() => call($15, "load8_u", [55_321]), 0);

// memory_copy.wast:2799
assert_return(() => call($15, "load8_u", [55_520]), 0);

// memory_copy.wast:2800
assert_return(() => call($15, "load8_u", [55_719]), 0);

// memory_copy.wast:2801
assert_return(() => call($15, "load8_u", [55_918]), 0);

// memory_copy.wast:2802
assert_return(() => call($15, "load8_u", [56_117]), 0);

// memory_copy.wast:2803
assert_return(() => call($15, "load8_u", [56_316]), 0);

// memory_copy.wast:2804
assert_return(() => call($15, "load8_u", [56_515]), 0);

// memory_copy.wast:2805
assert_return(() => call($15, "load8_u", [56_714]), 0);

// memory_copy.wast:2806
assert_return(() => call($15, "load8_u", [56_913]), 0);

// memory_copy.wast:2807
assert_return(() => call($15, "load8_u", [57_112]), 0);

// memory_copy.wast:2808
assert_return(() => call($15, "load8_u", [57_311]), 0);

// memory_copy.wast:2809
assert_return(() => call($15, "load8_u", [57_510]), 0);

// memory_copy.wast:2810
assert_return(() => call($15, "load8_u", [57_709]), 0);

// memory_copy.wast:2811
assert_return(() => call($15, "load8_u", [57_908]), 0);

// memory_copy.wast:2812
assert_return(() => call($15, "load8_u", [58_107]), 0);

// memory_copy.wast:2813
assert_return(() => call($15, "load8_u", [58_306]), 0);

// memory_copy.wast:2814
assert_return(() => call($15, "load8_u", [58_505]), 0);

// memory_copy.wast:2815
assert_return(() => call($15, "load8_u", [58_704]), 0);

// memory_copy.wast:2816
assert_return(() => call($15, "load8_u", [58_903]), 0);

// memory_copy.wast:2817
assert_return(() => call($15, "load8_u", [59_102]), 0);

// memory_copy.wast:2818
assert_return(() => call($15, "load8_u", [59_301]), 0);

// memory_copy.wast:2819
assert_return(() => call($15, "load8_u", [59_500]), 0);

// memory_copy.wast:2820
assert_return(() => call($15, "load8_u", [59_699]), 0);

// memory_copy.wast:2821
assert_return(() => call($15, "load8_u", [59_898]), 0);

// memory_copy.wast:2822
assert_return(() => call($15, "load8_u", [60_097]), 0);

// memory_copy.wast:2823
assert_return(() => call($15, "load8_u", [60_296]), 0);

// memory_copy.wast:2824
assert_return(() => call($15, "load8_u", [60_495]), 0);

// memory_copy.wast:2825
assert_return(() => call($15, "load8_u", [60_694]), 0);

// memory_copy.wast:2826
assert_return(() => call($15, "load8_u", [60_893]), 0);

// memory_copy.wast:2827
assert_return(() => call($15, "load8_u", [61_092]), 0);

// memory_copy.wast:2828
assert_return(() => call($15, "load8_u", [61_291]), 0);

// memory_copy.wast:2829
assert_return(() => call($15, "load8_u", [61_490]), 0);

// memory_copy.wast:2830
assert_return(() => call($15, "load8_u", [61_689]), 0);

// memory_copy.wast:2831
assert_return(() => call($15, "load8_u", [61_888]), 0);

// memory_copy.wast:2832
assert_return(() => call($15, "load8_u", [62_087]), 0);

// memory_copy.wast:2833
assert_return(() => call($15, "load8_u", [62_286]), 0);

// memory_copy.wast:2834
assert_return(() => call($15, "load8_u", [62_485]), 0);

// memory_copy.wast:2835
assert_return(() => call($15, "load8_u", [62_684]), 0);

// memory_copy.wast:2836
assert_return(() => call($15, "load8_u", [62_883]), 0);

// memory_copy.wast:2837
assert_return(() => call($15, "load8_u", [63_082]), 0);

// memory_copy.wast:2838
assert_return(() => call($15, "load8_u", [63_281]), 0);

// memory_copy.wast:2839
assert_return(() => call($15, "load8_u", [63_480]), 0);

// memory_copy.wast:2840
assert_return(() => call($15, "load8_u", [63_679]), 0);

// memory_copy.wast:2841
assert_return(() => call($15, "load8_u", [63_878]), 0);

// memory_copy.wast:2842
assert_return(() => call($15, "load8_u", [64_077]), 0);

// memory_copy.wast:2843
assert_return(() => call($15, "load8_u", [64_276]), 0);

// memory_copy.wast:2844
assert_return(() => call($15, "load8_u", [64_475]), 0);

// memory_copy.wast:2845
assert_return(() => call($15, "load8_u", [64_674]), 0);

// memory_copy.wast:2846
assert_return(() => call($15, "load8_u", [64_873]), 0);

// memory_copy.wast:2847
assert_return(() => call($15, "load8_u", [65_072]), 0);

// memory_copy.wast:2848
assert_return(() => call($15, "load8_u", [65_271]), 0);

// memory_copy.wast:2849
assert_return(() => call($15, "load8_u", [65_470]), 0);

// memory_copy.wast:2850
assert_return(() => call($15, "load8_u", [65_506]), 0);

// memory_copy.wast:2851
assert_return(() => call($15, "load8_u", [65_507]), 1);

// memory_copy.wast:2852
assert_return(() => call($15, "load8_u", [65_508]), 2);

// memory_copy.wast:2853
assert_return(() => call($15, "load8_u", [65_509]), 3);

// memory_copy.wast:2854
assert_return(() => call($15, "load8_u", [65_510]), 4);

// memory_copy.wast:2855
assert_return(() => call($15, "load8_u", [65_511]), 5);

// memory_copy.wast:2856
assert_return(() => call($15, "load8_u", [65_512]), 6);

// memory_copy.wast:2857
assert_return(() => call($15, "load8_u", [65_513]), 7);

// memory_copy.wast:2858
assert_return(() => call($15, "load8_u", [65_514]), 8);

// memory_copy.wast:2859
assert_return(() => call($15, "load8_u", [65_515]), 9);

// memory_copy.wast:2860
assert_return(() => call($15, "load8_u", [65_516]), 10);

// memory_copy.wast:2861
assert_return(() => call($15, "load8_u", [65_517]), 11);

// memory_copy.wast:2862
assert_return(() => call($15, "load8_u", [65_518]), 12);

// memory_copy.wast:2863
assert_return(() => call($15, "load8_u", [65_519]), 13);

// memory_copy.wast:2864
assert_return(() => call($15, "load8_u", [65_520]), 14);

// memory_copy.wast:2865
assert_return(() => call($15, "load8_u", [65_521]), 15);

// memory_copy.wast:2866
assert_return(() => call($15, "load8_u", [65_522]), 16);

// memory_copy.wast:2867
assert_return(() => call($15, "load8_u", [65_523]), 17);

// memory_copy.wast:2868
assert_return(() => call($15, "load8_u", [65_524]), 18);

// memory_copy.wast:2869
assert_return(() => call($15, "load8_u", [65_525]), 19);

// memory_copy.wast:2871
let $16 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9c\x80\x80\x80\x00\x01\x00\x41\xec\xff\x03\x0b\x14\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

// memory_copy.wast:2879
assert_trap(() => call($16, "run", [65_506, 65_516, 40]));

// memory_copy.wast:2882
assert_return(() => call($16, "load8_u", [198]), 0);

// memory_copy.wast:2883
assert_return(() => call($16, "load8_u", [397]), 0);

// memory_copy.wast:2884
assert_return(() => call($16, "load8_u", [596]), 0);

// memory_copy.wast:2885
assert_return(() => call($16, "load8_u", [795]), 0);

// memory_copy.wast:2886
assert_return(() => call($16, "load8_u", [994]), 0);

// memory_copy.wast:2887
assert_return(() => call($16, "load8_u", [1_193]), 0);

// memory_copy.wast:2888
assert_return(() => call($16, "load8_u", [1_392]), 0);

// memory_copy.wast:2889
assert_return(() => call($16, "load8_u", [1_591]), 0);

// memory_copy.wast:2890
assert_return(() => call($16, "load8_u", [1_790]), 0);

// memory_copy.wast:2891
assert_return(() => call($16, "load8_u", [1_989]), 0);

// memory_copy.wast:2892
assert_return(() => call($16, "load8_u", [2_188]), 0);

// memory_copy.wast:2893
assert_return(() => call($16, "load8_u", [2_387]), 0);

// memory_copy.wast:2894
assert_return(() => call($16, "load8_u", [2_586]), 0);

// memory_copy.wast:2895
assert_return(() => call($16, "load8_u", [2_785]), 0);

// memory_copy.wast:2896
assert_return(() => call($16, "load8_u", [2_984]), 0);

// memory_copy.wast:2897
assert_return(() => call($16, "load8_u", [3_183]), 0);

// memory_copy.wast:2898
assert_return(() => call($16, "load8_u", [3_382]), 0);

// memory_copy.wast:2899
assert_return(() => call($16, "load8_u", [3_581]), 0);

// memory_copy.wast:2900
assert_return(() => call($16, "load8_u", [3_780]), 0);

// memory_copy.wast:2901
assert_return(() => call($16, "load8_u", [3_979]), 0);

// memory_copy.wast:2902
assert_return(() => call($16, "load8_u", [4_178]), 0);

// memory_copy.wast:2903
assert_return(() => call($16, "load8_u", [4_377]), 0);

// memory_copy.wast:2904
assert_return(() => call($16, "load8_u", [4_576]), 0);

// memory_copy.wast:2905
assert_return(() => call($16, "load8_u", [4_775]), 0);

// memory_copy.wast:2906
assert_return(() => call($16, "load8_u", [4_974]), 0);

// memory_copy.wast:2907
assert_return(() => call($16, "load8_u", [5_173]), 0);

// memory_copy.wast:2908
assert_return(() => call($16, "load8_u", [5_372]), 0);

// memory_copy.wast:2909
assert_return(() => call($16, "load8_u", [5_571]), 0);

// memory_copy.wast:2910
assert_return(() => call($16, "load8_u", [5_770]), 0);

// memory_copy.wast:2911
assert_return(() => call($16, "load8_u", [5_969]), 0);

// memory_copy.wast:2912
assert_return(() => call($16, "load8_u", [6_168]), 0);

// memory_copy.wast:2913
assert_return(() => call($16, "load8_u", [6_367]), 0);

// memory_copy.wast:2914
assert_return(() => call($16, "load8_u", [6_566]), 0);

// memory_copy.wast:2915
assert_return(() => call($16, "load8_u", [6_765]), 0);

// memory_copy.wast:2916
assert_return(() => call($16, "load8_u", [6_964]), 0);

// memory_copy.wast:2917
assert_return(() => call($16, "load8_u", [7_163]), 0);

// memory_copy.wast:2918
assert_return(() => call($16, "load8_u", [7_362]), 0);

// memory_copy.wast:2919
assert_return(() => call($16, "load8_u", [7_561]), 0);

// memory_copy.wast:2920
assert_return(() => call($16, "load8_u", [7_760]), 0);

// memory_copy.wast:2921
assert_return(() => call($16, "load8_u", [7_959]), 0);

// memory_copy.wast:2922
assert_return(() => call($16, "load8_u", [8_158]), 0);

// memory_copy.wast:2923
assert_return(() => call($16, "load8_u", [8_357]), 0);

// memory_copy.wast:2924
assert_return(() => call($16, "load8_u", [8_556]), 0);

// memory_copy.wast:2925
assert_return(() => call($16, "load8_u", [8_755]), 0);

// memory_copy.wast:2926
assert_return(() => call($16, "load8_u", [8_954]), 0);

// memory_copy.wast:2927
assert_return(() => call($16, "load8_u", [9_153]), 0);

// memory_copy.wast:2928
assert_return(() => call($16, "load8_u", [9_352]), 0);

// memory_copy.wast:2929
assert_return(() => call($16, "load8_u", [9_551]), 0);

// memory_copy.wast:2930
assert_return(() => call($16, "load8_u", [9_750]), 0);

// memory_copy.wast:2931
assert_return(() => call($16, "load8_u", [9_949]), 0);

// memory_copy.wast:2932
assert_return(() => call($16, "load8_u", [10_148]), 0);

// memory_copy.wast:2933
assert_return(() => call($16, "load8_u", [10_347]), 0);

// memory_copy.wast:2934
assert_return(() => call($16, "load8_u", [10_546]), 0);

// memory_copy.wast:2935
assert_return(() => call($16, "load8_u", [10_745]), 0);

// memory_copy.wast:2936
assert_return(() => call($16, "load8_u", [10_944]), 0);

// memory_copy.wast:2937
assert_return(() => call($16, "load8_u", [11_143]), 0);

// memory_copy.wast:2938
assert_return(() => call($16, "load8_u", [11_342]), 0);

// memory_copy.wast:2939
assert_return(() => call($16, "load8_u", [11_541]), 0);

// memory_copy.wast:2940
assert_return(() => call($16, "load8_u", [11_740]), 0);

// memory_copy.wast:2941
assert_return(() => call($16, "load8_u", [11_939]), 0);

// memory_copy.wast:2942
assert_return(() => call($16, "load8_u", [12_138]), 0);

// memory_copy.wast:2943
assert_return(() => call($16, "load8_u", [12_337]), 0);

// memory_copy.wast:2944
assert_return(() => call($16, "load8_u", [12_536]), 0);

// memory_copy.wast:2945
assert_return(() => call($16, "load8_u", [12_735]), 0);

// memory_copy.wast:2946
assert_return(() => call($16, "load8_u", [12_934]), 0);

// memory_copy.wast:2947
assert_return(() => call($16, "load8_u", [13_133]), 0);

// memory_copy.wast:2948
assert_return(() => call($16, "load8_u", [13_332]), 0);

// memory_copy.wast:2949
assert_return(() => call($16, "load8_u", [13_531]), 0);

// memory_copy.wast:2950
assert_return(() => call($16, "load8_u", [13_730]), 0);

// memory_copy.wast:2951
assert_return(() => call($16, "load8_u", [13_929]), 0);

// memory_copy.wast:2952
assert_return(() => call($16, "load8_u", [14_128]), 0);

// memory_copy.wast:2953
assert_return(() => call($16, "load8_u", [14_327]), 0);

// memory_copy.wast:2954
assert_return(() => call($16, "load8_u", [14_526]), 0);

// memory_copy.wast:2955
assert_return(() => call($16, "load8_u", [14_725]), 0);

// memory_copy.wast:2956
assert_return(() => call($16, "load8_u", [14_924]), 0);

// memory_copy.wast:2957
assert_return(() => call($16, "load8_u", [15_123]), 0);

// memory_copy.wast:2958
assert_return(() => call($16, "load8_u", [15_322]), 0);

// memory_copy.wast:2959
assert_return(() => call($16, "load8_u", [15_521]), 0);

// memory_copy.wast:2960
assert_return(() => call($16, "load8_u", [15_720]), 0);

// memory_copy.wast:2961
assert_return(() => call($16, "load8_u", [15_919]), 0);

// memory_copy.wast:2962
assert_return(() => call($16, "load8_u", [16_118]), 0);

// memory_copy.wast:2963
assert_return(() => call($16, "load8_u", [16_317]), 0);

// memory_copy.wast:2964
assert_return(() => call($16, "load8_u", [16_516]), 0);

// memory_copy.wast:2965
assert_return(() => call($16, "load8_u", [16_715]), 0);

// memory_copy.wast:2966
assert_return(() => call($16, "load8_u", [16_914]), 0);

// memory_copy.wast:2967
assert_return(() => call($16, "load8_u", [17_113]), 0);

// memory_copy.wast:2968
assert_return(() => call($16, "load8_u", [17_312]), 0);

// memory_copy.wast:2969
assert_return(() => call($16, "load8_u", [17_511]), 0);

// memory_copy.wast:2970
assert_return(() => call($16, "load8_u", [17_710]), 0);

// memory_copy.wast:2971
assert_return(() => call($16, "load8_u", [17_909]), 0);

// memory_copy.wast:2972
assert_return(() => call($16, "load8_u", [18_108]), 0);

// memory_copy.wast:2973
assert_return(() => call($16, "load8_u", [18_307]), 0);

// memory_copy.wast:2974
assert_return(() => call($16, "load8_u", [18_506]), 0);

// memory_copy.wast:2975
assert_return(() => call($16, "load8_u", [18_705]), 0);

// memory_copy.wast:2976
assert_return(() => call($16, "load8_u", [18_904]), 0);

// memory_copy.wast:2977
assert_return(() => call($16, "load8_u", [19_103]), 0);

// memory_copy.wast:2978
assert_return(() => call($16, "load8_u", [19_302]), 0);

// memory_copy.wast:2979
assert_return(() => call($16, "load8_u", [19_501]), 0);

// memory_copy.wast:2980
assert_return(() => call($16, "load8_u", [19_700]), 0);

// memory_copy.wast:2981
assert_return(() => call($16, "load8_u", [19_899]), 0);

// memory_copy.wast:2982
assert_return(() => call($16, "load8_u", [20_098]), 0);

// memory_copy.wast:2983
assert_return(() => call($16, "load8_u", [20_297]), 0);

// memory_copy.wast:2984
assert_return(() => call($16, "load8_u", [20_496]), 0);

// memory_copy.wast:2985
assert_return(() => call($16, "load8_u", [20_695]), 0);

// memory_copy.wast:2986
assert_return(() => call($16, "load8_u", [20_894]), 0);

// memory_copy.wast:2987
assert_return(() => call($16, "load8_u", [21_093]), 0);

// memory_copy.wast:2988
assert_return(() => call($16, "load8_u", [21_292]), 0);

// memory_copy.wast:2989
assert_return(() => call($16, "load8_u", [21_491]), 0);

// memory_copy.wast:2990
assert_return(() => call($16, "load8_u", [21_690]), 0);

// memory_copy.wast:2991
assert_return(() => call($16, "load8_u", [21_889]), 0);

// memory_copy.wast:2992
assert_return(() => call($16, "load8_u", [22_088]), 0);

// memory_copy.wast:2993
assert_return(() => call($16, "load8_u", [22_287]), 0);

// memory_copy.wast:2994
assert_return(() => call($16, "load8_u", [22_486]), 0);

// memory_copy.wast:2995
assert_return(() => call($16, "load8_u", [22_685]), 0);

// memory_copy.wast:2996
assert_return(() => call($16, "load8_u", [22_884]), 0);

// memory_copy.wast:2997
assert_return(() => call($16, "load8_u", [23_083]), 0);

// memory_copy.wast:2998
assert_return(() => call($16, "load8_u", [23_282]), 0);

// memory_copy.wast:2999
assert_return(() => call($16, "load8_u", [23_481]), 0);

// memory_copy.wast:3000
assert_return(() => call($16, "load8_u", [23_680]), 0);

// memory_copy.wast:3001
assert_return(() => call($16, "load8_u", [23_879]), 0);

// memory_copy.wast:3002
assert_return(() => call($16, "load8_u", [24_078]), 0);

// memory_copy.wast:3003
assert_return(() => call($16, "load8_u", [24_277]), 0);

// memory_copy.wast:3004
assert_return(() => call($16, "load8_u", [24_476]), 0);

// memory_copy.wast:3005
assert_return(() => call($16, "load8_u", [24_675]), 0);

// memory_copy.wast:3006
assert_return(() => call($16, "load8_u", [24_874]), 0);

// memory_copy.wast:3007
assert_return(() => call($16, "load8_u", [25_073]), 0);

// memory_copy.wast:3008
assert_return(() => call($16, "load8_u", [25_272]), 0);

// memory_copy.wast:3009
assert_return(() => call($16, "load8_u", [25_471]), 0);

// memory_copy.wast:3010
assert_return(() => call($16, "load8_u", [25_670]), 0);

// memory_copy.wast:3011
assert_return(() => call($16, "load8_u", [25_869]), 0);

// memory_copy.wast:3012
assert_return(() => call($16, "load8_u", [26_068]), 0);

// memory_copy.wast:3013
assert_return(() => call($16, "load8_u", [26_267]), 0);

// memory_copy.wast:3014
assert_return(() => call($16, "load8_u", [26_466]), 0);

// memory_copy.wast:3015
assert_return(() => call($16, "load8_u", [26_665]), 0);

// memory_copy.wast:3016
assert_return(() => call($16, "load8_u", [26_864]), 0);

// memory_copy.wast:3017
assert_return(() => call($16, "load8_u", [27_063]), 0);

// memory_copy.wast:3018
assert_return(() => call($16, "load8_u", [27_262]), 0);

// memory_copy.wast:3019
assert_return(() => call($16, "load8_u", [27_461]), 0);

// memory_copy.wast:3020
assert_return(() => call($16, "load8_u", [27_660]), 0);

// memory_copy.wast:3021
assert_return(() => call($16, "load8_u", [27_859]), 0);

// memory_copy.wast:3022
assert_return(() => call($16, "load8_u", [28_058]), 0);

// memory_copy.wast:3023
assert_return(() => call($16, "load8_u", [28_257]), 0);

// memory_copy.wast:3024
assert_return(() => call($16, "load8_u", [28_456]), 0);

// memory_copy.wast:3025
assert_return(() => call($16, "load8_u", [28_655]), 0);

// memory_copy.wast:3026
assert_return(() => call($16, "load8_u", [28_854]), 0);

// memory_copy.wast:3027
assert_return(() => call($16, "load8_u", [29_053]), 0);

// memory_copy.wast:3028
assert_return(() => call($16, "load8_u", [29_252]), 0);

// memory_copy.wast:3029
assert_return(() => call($16, "load8_u", [29_451]), 0);

// memory_copy.wast:3030
assert_return(() => call($16, "load8_u", [29_650]), 0);

// memory_copy.wast:3031
assert_return(() => call($16, "load8_u", [29_849]), 0);

// memory_copy.wast:3032
assert_return(() => call($16, "load8_u", [30_048]), 0);

// memory_copy.wast:3033
assert_return(() => call($16, "load8_u", [30_247]), 0);

// memory_copy.wast:3034
assert_return(() => call($16, "load8_u", [30_446]), 0);

// memory_copy.wast:3035
assert_return(() => call($16, "load8_u", [30_645]), 0);

// memory_copy.wast:3036
assert_return(() => call($16, "load8_u", [30_844]), 0);

// memory_copy.wast:3037
assert_return(() => call($16, "load8_u", [31_043]), 0);

// memory_copy.wast:3038
assert_return(() => call($16, "load8_u", [31_242]), 0);

// memory_copy.wast:3039
assert_return(() => call($16, "load8_u", [31_441]), 0);

// memory_copy.wast:3040
assert_return(() => call($16, "load8_u", [31_640]), 0);

// memory_copy.wast:3041
assert_return(() => call($16, "load8_u", [31_839]), 0);

// memory_copy.wast:3042
assert_return(() => call($16, "load8_u", [32_038]), 0);

// memory_copy.wast:3043
assert_return(() => call($16, "load8_u", [32_237]), 0);

// memory_copy.wast:3044
assert_return(() => call($16, "load8_u", [32_436]), 0);

// memory_copy.wast:3045
assert_return(() => call($16, "load8_u", [32_635]), 0);

// memory_copy.wast:3046
assert_return(() => call($16, "load8_u", [32_834]), 0);

// memory_copy.wast:3047
assert_return(() => call($16, "load8_u", [33_033]), 0);

// memory_copy.wast:3048
assert_return(() => call($16, "load8_u", [33_232]), 0);

// memory_copy.wast:3049
assert_return(() => call($16, "load8_u", [33_431]), 0);

// memory_copy.wast:3050
assert_return(() => call($16, "load8_u", [33_630]), 0);

// memory_copy.wast:3051
assert_return(() => call($16, "load8_u", [33_829]), 0);

// memory_copy.wast:3052
assert_return(() => call($16, "load8_u", [34_028]), 0);

// memory_copy.wast:3053
assert_return(() => call($16, "load8_u", [34_227]), 0);

// memory_copy.wast:3054
assert_return(() => call($16, "load8_u", [34_426]), 0);

// memory_copy.wast:3055
assert_return(() => call($16, "load8_u", [34_625]), 0);

// memory_copy.wast:3056
assert_return(() => call($16, "load8_u", [34_824]), 0);

// memory_copy.wast:3057
assert_return(() => call($16, "load8_u", [35_023]), 0);

// memory_copy.wast:3058
assert_return(() => call($16, "load8_u", [35_222]), 0);

// memory_copy.wast:3059
assert_return(() => call($16, "load8_u", [35_421]), 0);

// memory_copy.wast:3060
assert_return(() => call($16, "load8_u", [35_620]), 0);

// memory_copy.wast:3061
assert_return(() => call($16, "load8_u", [35_819]), 0);

// memory_copy.wast:3062
assert_return(() => call($16, "load8_u", [36_018]), 0);

// memory_copy.wast:3063
assert_return(() => call($16, "load8_u", [36_217]), 0);

// memory_copy.wast:3064
assert_return(() => call($16, "load8_u", [36_416]), 0);

// memory_copy.wast:3065
assert_return(() => call($16, "load8_u", [36_615]), 0);

// memory_copy.wast:3066
assert_return(() => call($16, "load8_u", [36_814]), 0);

// memory_copy.wast:3067
assert_return(() => call($16, "load8_u", [37_013]), 0);

// memory_copy.wast:3068
assert_return(() => call($16, "load8_u", [37_212]), 0);

// memory_copy.wast:3069
assert_return(() => call($16, "load8_u", [37_411]), 0);

// memory_copy.wast:3070
assert_return(() => call($16, "load8_u", [37_610]), 0);

// memory_copy.wast:3071
assert_return(() => call($16, "load8_u", [37_809]), 0);

// memory_copy.wast:3072
assert_return(() => call($16, "load8_u", [38_008]), 0);

// memory_copy.wast:3073
assert_return(() => call($16, "load8_u", [38_207]), 0);

// memory_copy.wast:3074
assert_return(() => call($16, "load8_u", [38_406]), 0);

// memory_copy.wast:3075
assert_return(() => call($16, "load8_u", [38_605]), 0);

// memory_copy.wast:3076
assert_return(() => call($16, "load8_u", [38_804]), 0);

// memory_copy.wast:3077
assert_return(() => call($16, "load8_u", [39_003]), 0);

// memory_copy.wast:3078
assert_return(() => call($16, "load8_u", [39_202]), 0);

// memory_copy.wast:3079
assert_return(() => call($16, "load8_u", [39_401]), 0);

// memory_copy.wast:3080
assert_return(() => call($16, "load8_u", [39_600]), 0);

// memory_copy.wast:3081
assert_return(() => call($16, "load8_u", [39_799]), 0);

// memory_copy.wast:3082
assert_return(() => call($16, "load8_u", [39_998]), 0);

// memory_copy.wast:3083
assert_return(() => call($16, "load8_u", [40_197]), 0);

// memory_copy.wast:3084
assert_return(() => call($16, "load8_u", [40_396]), 0);

// memory_copy.wast:3085
assert_return(() => call($16, "load8_u", [40_595]), 0);

// memory_copy.wast:3086
assert_return(() => call($16, "load8_u", [40_794]), 0);

// memory_copy.wast:3087
assert_return(() => call($16, "load8_u", [40_993]), 0);

// memory_copy.wast:3088
assert_return(() => call($16, "load8_u", [41_192]), 0);

// memory_copy.wast:3089
assert_return(() => call($16, "load8_u", [41_391]), 0);

// memory_copy.wast:3090
assert_return(() => call($16, "load8_u", [41_590]), 0);

// memory_copy.wast:3091
assert_return(() => call($16, "load8_u", [41_789]), 0);

// memory_copy.wast:3092
assert_return(() => call($16, "load8_u", [41_988]), 0);

// memory_copy.wast:3093
assert_return(() => call($16, "load8_u", [42_187]), 0);

// memory_copy.wast:3094
assert_return(() => call($16, "load8_u", [42_386]), 0);

// memory_copy.wast:3095
assert_return(() => call($16, "load8_u", [42_585]), 0);

// memory_copy.wast:3096
assert_return(() => call($16, "load8_u", [42_784]), 0);

// memory_copy.wast:3097
assert_return(() => call($16, "load8_u", [42_983]), 0);

// memory_copy.wast:3098
assert_return(() => call($16, "load8_u", [43_182]), 0);

// memory_copy.wast:3099
assert_return(() => call($16, "load8_u", [43_381]), 0);

// memory_copy.wast:3100
assert_return(() => call($16, "load8_u", [43_580]), 0);

// memory_copy.wast:3101
assert_return(() => call($16, "load8_u", [43_779]), 0);

// memory_copy.wast:3102
assert_return(() => call($16, "load8_u", [43_978]), 0);

// memory_copy.wast:3103
assert_return(() => call($16, "load8_u", [44_177]), 0);

// memory_copy.wast:3104
assert_return(() => call($16, "load8_u", [44_376]), 0);

// memory_copy.wast:3105
assert_return(() => call($16, "load8_u", [44_575]), 0);

// memory_copy.wast:3106
assert_return(() => call($16, "load8_u", [44_774]), 0);

// memory_copy.wast:3107
assert_return(() => call($16, "load8_u", [44_973]), 0);

// memory_copy.wast:3108
assert_return(() => call($16, "load8_u", [45_172]), 0);

// memory_copy.wast:3109
assert_return(() => call($16, "load8_u", [45_371]), 0);

// memory_copy.wast:3110
assert_return(() => call($16, "load8_u", [45_570]), 0);

// memory_copy.wast:3111
assert_return(() => call($16, "load8_u", [45_769]), 0);

// memory_copy.wast:3112
assert_return(() => call($16, "load8_u", [45_968]), 0);

// memory_copy.wast:3113
assert_return(() => call($16, "load8_u", [46_167]), 0);

// memory_copy.wast:3114
assert_return(() => call($16, "load8_u", [46_366]), 0);

// memory_copy.wast:3115
assert_return(() => call($16, "load8_u", [46_565]), 0);

// memory_copy.wast:3116
assert_return(() => call($16, "load8_u", [46_764]), 0);

// memory_copy.wast:3117
assert_return(() => call($16, "load8_u", [46_963]), 0);

// memory_copy.wast:3118
assert_return(() => call($16, "load8_u", [47_162]), 0);

// memory_copy.wast:3119
assert_return(() => call($16, "load8_u", [47_361]), 0);

// memory_copy.wast:3120
assert_return(() => call($16, "load8_u", [47_560]), 0);

// memory_copy.wast:3121
assert_return(() => call($16, "load8_u", [47_759]), 0);

// memory_copy.wast:3122
assert_return(() => call($16, "load8_u", [47_958]), 0);

// memory_copy.wast:3123
assert_return(() => call($16, "load8_u", [48_157]), 0);

// memory_copy.wast:3124
assert_return(() => call($16, "load8_u", [48_356]), 0);

// memory_copy.wast:3125
assert_return(() => call($16, "load8_u", [48_555]), 0);

// memory_copy.wast:3126
assert_return(() => call($16, "load8_u", [48_754]), 0);

// memory_copy.wast:3127
assert_return(() => call($16, "load8_u", [48_953]), 0);

// memory_copy.wast:3128
assert_return(() => call($16, "load8_u", [49_152]), 0);

// memory_copy.wast:3129
assert_return(() => call($16, "load8_u", [49_351]), 0);

// memory_copy.wast:3130
assert_return(() => call($16, "load8_u", [49_550]), 0);

// memory_copy.wast:3131
assert_return(() => call($16, "load8_u", [49_749]), 0);

// memory_copy.wast:3132
assert_return(() => call($16, "load8_u", [49_948]), 0);

// memory_copy.wast:3133
assert_return(() => call($16, "load8_u", [50_147]), 0);

// memory_copy.wast:3134
assert_return(() => call($16, "load8_u", [50_346]), 0);

// memory_copy.wast:3135
assert_return(() => call($16, "load8_u", [50_545]), 0);

// memory_copy.wast:3136
assert_return(() => call($16, "load8_u", [50_744]), 0);

// memory_copy.wast:3137
assert_return(() => call($16, "load8_u", [50_943]), 0);

// memory_copy.wast:3138
assert_return(() => call($16, "load8_u", [51_142]), 0);

// memory_copy.wast:3139
assert_return(() => call($16, "load8_u", [51_341]), 0);

// memory_copy.wast:3140
assert_return(() => call($16, "load8_u", [51_540]), 0);

// memory_copy.wast:3141
assert_return(() => call($16, "load8_u", [51_739]), 0);

// memory_copy.wast:3142
assert_return(() => call($16, "load8_u", [51_938]), 0);

// memory_copy.wast:3143
assert_return(() => call($16, "load8_u", [52_137]), 0);

// memory_copy.wast:3144
assert_return(() => call($16, "load8_u", [52_336]), 0);

// memory_copy.wast:3145
assert_return(() => call($16, "load8_u", [52_535]), 0);

// memory_copy.wast:3146
assert_return(() => call($16, "load8_u", [52_734]), 0);

// memory_copy.wast:3147
assert_return(() => call($16, "load8_u", [52_933]), 0);

// memory_copy.wast:3148
assert_return(() => call($16, "load8_u", [53_132]), 0);

// memory_copy.wast:3149
assert_return(() => call($16, "load8_u", [53_331]), 0);

// memory_copy.wast:3150
assert_return(() => call($16, "load8_u", [53_530]), 0);

// memory_copy.wast:3151
assert_return(() => call($16, "load8_u", [53_729]), 0);

// memory_copy.wast:3152
assert_return(() => call($16, "load8_u", [53_928]), 0);

// memory_copy.wast:3153
assert_return(() => call($16, "load8_u", [54_127]), 0);

// memory_copy.wast:3154
assert_return(() => call($16, "load8_u", [54_326]), 0);

// memory_copy.wast:3155
assert_return(() => call($16, "load8_u", [54_525]), 0);

// memory_copy.wast:3156
assert_return(() => call($16, "load8_u", [54_724]), 0);

// memory_copy.wast:3157
assert_return(() => call($16, "load8_u", [54_923]), 0);

// memory_copy.wast:3158
assert_return(() => call($16, "load8_u", [55_122]), 0);

// memory_copy.wast:3159
assert_return(() => call($16, "load8_u", [55_321]), 0);

// memory_copy.wast:3160
assert_return(() => call($16, "load8_u", [55_520]), 0);

// memory_copy.wast:3161
assert_return(() => call($16, "load8_u", [55_719]), 0);

// memory_copy.wast:3162
assert_return(() => call($16, "load8_u", [55_918]), 0);

// memory_copy.wast:3163
assert_return(() => call($16, "load8_u", [56_117]), 0);

// memory_copy.wast:3164
assert_return(() => call($16, "load8_u", [56_316]), 0);

// memory_copy.wast:3165
assert_return(() => call($16, "load8_u", [56_515]), 0);

// memory_copy.wast:3166
assert_return(() => call($16, "load8_u", [56_714]), 0);

// memory_copy.wast:3167
assert_return(() => call($16, "load8_u", [56_913]), 0);

// memory_copy.wast:3168
assert_return(() => call($16, "load8_u", [57_112]), 0);

// memory_copy.wast:3169
assert_return(() => call($16, "load8_u", [57_311]), 0);

// memory_copy.wast:3170
assert_return(() => call($16, "load8_u", [57_510]), 0);

// memory_copy.wast:3171
assert_return(() => call($16, "load8_u", [57_709]), 0);

// memory_copy.wast:3172
assert_return(() => call($16, "load8_u", [57_908]), 0);

// memory_copy.wast:3173
assert_return(() => call($16, "load8_u", [58_107]), 0);

// memory_copy.wast:3174
assert_return(() => call($16, "load8_u", [58_306]), 0);

// memory_copy.wast:3175
assert_return(() => call($16, "load8_u", [58_505]), 0);

// memory_copy.wast:3176
assert_return(() => call($16, "load8_u", [58_704]), 0);

// memory_copy.wast:3177
assert_return(() => call($16, "load8_u", [58_903]), 0);

// memory_copy.wast:3178
assert_return(() => call($16, "load8_u", [59_102]), 0);

// memory_copy.wast:3179
assert_return(() => call($16, "load8_u", [59_301]), 0);

// memory_copy.wast:3180
assert_return(() => call($16, "load8_u", [59_500]), 0);

// memory_copy.wast:3181
assert_return(() => call($16, "load8_u", [59_699]), 0);

// memory_copy.wast:3182
assert_return(() => call($16, "load8_u", [59_898]), 0);

// memory_copy.wast:3183
assert_return(() => call($16, "load8_u", [60_097]), 0);

// memory_copy.wast:3184
assert_return(() => call($16, "load8_u", [60_296]), 0);

// memory_copy.wast:3185
assert_return(() => call($16, "load8_u", [60_495]), 0);

// memory_copy.wast:3186
assert_return(() => call($16, "load8_u", [60_694]), 0);

// memory_copy.wast:3187
assert_return(() => call($16, "load8_u", [60_893]), 0);

// memory_copy.wast:3188
assert_return(() => call($16, "load8_u", [61_092]), 0);

// memory_copy.wast:3189
assert_return(() => call($16, "load8_u", [61_291]), 0);

// memory_copy.wast:3190
assert_return(() => call($16, "load8_u", [61_490]), 0);

// memory_copy.wast:3191
assert_return(() => call($16, "load8_u", [61_689]), 0);

// memory_copy.wast:3192
assert_return(() => call($16, "load8_u", [61_888]), 0);

// memory_copy.wast:3193
assert_return(() => call($16, "load8_u", [62_087]), 0);

// memory_copy.wast:3194
assert_return(() => call($16, "load8_u", [62_286]), 0);

// memory_copy.wast:3195
assert_return(() => call($16, "load8_u", [62_485]), 0);

// memory_copy.wast:3196
assert_return(() => call($16, "load8_u", [62_684]), 0);

// memory_copy.wast:3197
assert_return(() => call($16, "load8_u", [62_883]), 0);

// memory_copy.wast:3198
assert_return(() => call($16, "load8_u", [63_082]), 0);

// memory_copy.wast:3199
assert_return(() => call($16, "load8_u", [63_281]), 0);

// memory_copy.wast:3200
assert_return(() => call($16, "load8_u", [63_480]), 0);

// memory_copy.wast:3201
assert_return(() => call($16, "load8_u", [63_679]), 0);

// memory_copy.wast:3202
assert_return(() => call($16, "load8_u", [63_878]), 0);

// memory_copy.wast:3203
assert_return(() => call($16, "load8_u", [64_077]), 0);

// memory_copy.wast:3204
assert_return(() => call($16, "load8_u", [64_276]), 0);

// memory_copy.wast:3205
assert_return(() => call($16, "load8_u", [64_475]), 0);

// memory_copy.wast:3206
assert_return(() => call($16, "load8_u", [64_674]), 0);

// memory_copy.wast:3207
assert_return(() => call($16, "load8_u", [64_873]), 0);

// memory_copy.wast:3208
assert_return(() => call($16, "load8_u", [65_072]), 0);

// memory_copy.wast:3209
assert_return(() => call($16, "load8_u", [65_271]), 0);

// memory_copy.wast:3210
assert_return(() => call($16, "load8_u", [65_470]), 0);

// memory_copy.wast:3211
assert_return(() => call($16, "load8_u", [65_516]), 0);

// memory_copy.wast:3212
assert_return(() => call($16, "load8_u", [65_517]), 1);

// memory_copy.wast:3213
assert_return(() => call($16, "load8_u", [65_518]), 2);

// memory_copy.wast:3214
assert_return(() => call($16, "load8_u", [65_519]), 3);

// memory_copy.wast:3215
assert_return(() => call($16, "load8_u", [65_520]), 4);

// memory_copy.wast:3216
assert_return(() => call($16, "load8_u", [65_521]), 5);

// memory_copy.wast:3217
assert_return(() => call($16, "load8_u", [65_522]), 6);

// memory_copy.wast:3218
assert_return(() => call($16, "load8_u", [65_523]), 7);

// memory_copy.wast:3219
assert_return(() => call($16, "load8_u", [65_524]), 8);

// memory_copy.wast:3220
assert_return(() => call($16, "load8_u", [65_525]), 9);

// memory_copy.wast:3221
assert_return(() => call($16, "load8_u", [65_526]), 10);

// memory_copy.wast:3222
assert_return(() => call($16, "load8_u", [65_527]), 11);

// memory_copy.wast:3223
assert_return(() => call($16, "load8_u", [65_528]), 12);

// memory_copy.wast:3224
assert_return(() => call($16, "load8_u", [65_529]), 13);

// memory_copy.wast:3225
assert_return(() => call($16, "load8_u", [65_530]), 14);

// memory_copy.wast:3226
assert_return(() => call($16, "load8_u", [65_531]), 15);

// memory_copy.wast:3227
assert_return(() => call($16, "load8_u", [65_532]), 16);

// memory_copy.wast:3228
assert_return(() => call($16, "load8_u", [65_533]), 17);

// memory_copy.wast:3229
assert_return(() => call($16, "load8_u", [65_534]), 18);

// memory_copy.wast:3230
assert_return(() => call($16, "load8_u", [65_535]), 19);

// memory_copy.wast:3232
let $17 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9c\x80\x80\x80\x00\x01\x00\x41\xec\xff\x03\x0b\x14\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

// memory_copy.wast:3240
assert_trap(() => call($17, "run", [65_516, 65_516, 40]));

// memory_copy.wast:3243
assert_return(() => call($17, "load8_u", [198]), 0);

// memory_copy.wast:3244
assert_return(() => call($17, "load8_u", [397]), 0);

// memory_copy.wast:3245
assert_return(() => call($17, "load8_u", [596]), 0);

// memory_copy.wast:3246
assert_return(() => call($17, "load8_u", [795]), 0);

// memory_copy.wast:3247
assert_return(() => call($17, "load8_u", [994]), 0);

// memory_copy.wast:3248
assert_return(() => call($17, "load8_u", [1_193]), 0);

// memory_copy.wast:3249
assert_return(() => call($17, "load8_u", [1_392]), 0);

// memory_copy.wast:3250
assert_return(() => call($17, "load8_u", [1_591]), 0);

// memory_copy.wast:3251
assert_return(() => call($17, "load8_u", [1_790]), 0);

// memory_copy.wast:3252
assert_return(() => call($17, "load8_u", [1_989]), 0);

// memory_copy.wast:3253
assert_return(() => call($17, "load8_u", [2_188]), 0);

// memory_copy.wast:3254
assert_return(() => call($17, "load8_u", [2_387]), 0);

// memory_copy.wast:3255
assert_return(() => call($17, "load8_u", [2_586]), 0);

// memory_copy.wast:3256
assert_return(() => call($17, "load8_u", [2_785]), 0);

// memory_copy.wast:3257
assert_return(() => call($17, "load8_u", [2_984]), 0);

// memory_copy.wast:3258
assert_return(() => call($17, "load8_u", [3_183]), 0);

// memory_copy.wast:3259
assert_return(() => call($17, "load8_u", [3_382]), 0);

// memory_copy.wast:3260
assert_return(() => call($17, "load8_u", [3_581]), 0);

// memory_copy.wast:3261
assert_return(() => call($17, "load8_u", [3_780]), 0);

// memory_copy.wast:3262
assert_return(() => call($17, "load8_u", [3_979]), 0);

// memory_copy.wast:3263
assert_return(() => call($17, "load8_u", [4_178]), 0);

// memory_copy.wast:3264
assert_return(() => call($17, "load8_u", [4_377]), 0);

// memory_copy.wast:3265
assert_return(() => call($17, "load8_u", [4_576]), 0);

// memory_copy.wast:3266
assert_return(() => call($17, "load8_u", [4_775]), 0);

// memory_copy.wast:3267
assert_return(() => call($17, "load8_u", [4_974]), 0);

// memory_copy.wast:3268
assert_return(() => call($17, "load8_u", [5_173]), 0);

// memory_copy.wast:3269
assert_return(() => call($17, "load8_u", [5_372]), 0);

// memory_copy.wast:3270
assert_return(() => call($17, "load8_u", [5_571]), 0);

// memory_copy.wast:3271
assert_return(() => call($17, "load8_u", [5_770]), 0);

// memory_copy.wast:3272
assert_return(() => call($17, "load8_u", [5_969]), 0);

// memory_copy.wast:3273
assert_return(() => call($17, "load8_u", [6_168]), 0);

// memory_copy.wast:3274
assert_return(() => call($17, "load8_u", [6_367]), 0);

// memory_copy.wast:3275
assert_return(() => call($17, "load8_u", [6_566]), 0);

// memory_copy.wast:3276
assert_return(() => call($17, "load8_u", [6_765]), 0);

// memory_copy.wast:3277
assert_return(() => call($17, "load8_u", [6_964]), 0);

// memory_copy.wast:3278
assert_return(() => call($17, "load8_u", [7_163]), 0);

// memory_copy.wast:3279
assert_return(() => call($17, "load8_u", [7_362]), 0);

// memory_copy.wast:3280
assert_return(() => call($17, "load8_u", [7_561]), 0);

// memory_copy.wast:3281
assert_return(() => call($17, "load8_u", [7_760]), 0);

// memory_copy.wast:3282
assert_return(() => call($17, "load8_u", [7_959]), 0);

// memory_copy.wast:3283
assert_return(() => call($17, "load8_u", [8_158]), 0);

// memory_copy.wast:3284
assert_return(() => call($17, "load8_u", [8_357]), 0);

// memory_copy.wast:3285
assert_return(() => call($17, "load8_u", [8_556]), 0);

// memory_copy.wast:3286
assert_return(() => call($17, "load8_u", [8_755]), 0);

// memory_copy.wast:3287
assert_return(() => call($17, "load8_u", [8_954]), 0);

// memory_copy.wast:3288
assert_return(() => call($17, "load8_u", [9_153]), 0);

// memory_copy.wast:3289
assert_return(() => call($17, "load8_u", [9_352]), 0);

// memory_copy.wast:3290
assert_return(() => call($17, "load8_u", [9_551]), 0);

// memory_copy.wast:3291
assert_return(() => call($17, "load8_u", [9_750]), 0);

// memory_copy.wast:3292
assert_return(() => call($17, "load8_u", [9_949]), 0);

// memory_copy.wast:3293
assert_return(() => call($17, "load8_u", [10_148]), 0);

// memory_copy.wast:3294
assert_return(() => call($17, "load8_u", [10_347]), 0);

// memory_copy.wast:3295
assert_return(() => call($17, "load8_u", [10_546]), 0);

// memory_copy.wast:3296
assert_return(() => call($17, "load8_u", [10_745]), 0);

// memory_copy.wast:3297
assert_return(() => call($17, "load8_u", [10_944]), 0);

// memory_copy.wast:3298
assert_return(() => call($17, "load8_u", [11_143]), 0);

// memory_copy.wast:3299
assert_return(() => call($17, "load8_u", [11_342]), 0);

// memory_copy.wast:3300
assert_return(() => call($17, "load8_u", [11_541]), 0);

// memory_copy.wast:3301
assert_return(() => call($17, "load8_u", [11_740]), 0);

// memory_copy.wast:3302
assert_return(() => call($17, "load8_u", [11_939]), 0);

// memory_copy.wast:3303
assert_return(() => call($17, "load8_u", [12_138]), 0);

// memory_copy.wast:3304
assert_return(() => call($17, "load8_u", [12_337]), 0);

// memory_copy.wast:3305
assert_return(() => call($17, "load8_u", [12_536]), 0);

// memory_copy.wast:3306
assert_return(() => call($17, "load8_u", [12_735]), 0);

// memory_copy.wast:3307
assert_return(() => call($17, "load8_u", [12_934]), 0);

// memory_copy.wast:3308
assert_return(() => call($17, "load8_u", [13_133]), 0);

// memory_copy.wast:3309
assert_return(() => call($17, "load8_u", [13_332]), 0);

// memory_copy.wast:3310
assert_return(() => call($17, "load8_u", [13_531]), 0);

// memory_copy.wast:3311
assert_return(() => call($17, "load8_u", [13_730]), 0);

// memory_copy.wast:3312
assert_return(() => call($17, "load8_u", [13_929]), 0);

// memory_copy.wast:3313
assert_return(() => call($17, "load8_u", [14_128]), 0);

// memory_copy.wast:3314
assert_return(() => call($17, "load8_u", [14_327]), 0);

// memory_copy.wast:3315
assert_return(() => call($17, "load8_u", [14_526]), 0);

// memory_copy.wast:3316
assert_return(() => call($17, "load8_u", [14_725]), 0);

// memory_copy.wast:3317
assert_return(() => call($17, "load8_u", [14_924]), 0);

// memory_copy.wast:3318
assert_return(() => call($17, "load8_u", [15_123]), 0);

// memory_copy.wast:3319
assert_return(() => call($17, "load8_u", [15_322]), 0);

// memory_copy.wast:3320
assert_return(() => call($17, "load8_u", [15_521]), 0);

// memory_copy.wast:3321
assert_return(() => call($17, "load8_u", [15_720]), 0);

// memory_copy.wast:3322
assert_return(() => call($17, "load8_u", [15_919]), 0);

// memory_copy.wast:3323
assert_return(() => call($17, "load8_u", [16_118]), 0);

// memory_copy.wast:3324
assert_return(() => call($17, "load8_u", [16_317]), 0);

// memory_copy.wast:3325
assert_return(() => call($17, "load8_u", [16_516]), 0);

// memory_copy.wast:3326
assert_return(() => call($17, "load8_u", [16_715]), 0);

// memory_copy.wast:3327
assert_return(() => call($17, "load8_u", [16_914]), 0);

// memory_copy.wast:3328
assert_return(() => call($17, "load8_u", [17_113]), 0);

// memory_copy.wast:3329
assert_return(() => call($17, "load8_u", [17_312]), 0);

// memory_copy.wast:3330
assert_return(() => call($17, "load8_u", [17_511]), 0);

// memory_copy.wast:3331
assert_return(() => call($17, "load8_u", [17_710]), 0);

// memory_copy.wast:3332
assert_return(() => call($17, "load8_u", [17_909]), 0);

// memory_copy.wast:3333
assert_return(() => call($17, "load8_u", [18_108]), 0);

// memory_copy.wast:3334
assert_return(() => call($17, "load8_u", [18_307]), 0);

// memory_copy.wast:3335
assert_return(() => call($17, "load8_u", [18_506]), 0);

// memory_copy.wast:3336
assert_return(() => call($17, "load8_u", [18_705]), 0);

// memory_copy.wast:3337
assert_return(() => call($17, "load8_u", [18_904]), 0);

// memory_copy.wast:3338
assert_return(() => call($17, "load8_u", [19_103]), 0);

// memory_copy.wast:3339
assert_return(() => call($17, "load8_u", [19_302]), 0);

// memory_copy.wast:3340
assert_return(() => call($17, "load8_u", [19_501]), 0);

// memory_copy.wast:3341
assert_return(() => call($17, "load8_u", [19_700]), 0);

// memory_copy.wast:3342
assert_return(() => call($17, "load8_u", [19_899]), 0);

// memory_copy.wast:3343
assert_return(() => call($17, "load8_u", [20_098]), 0);

// memory_copy.wast:3344
assert_return(() => call($17, "load8_u", [20_297]), 0);

// memory_copy.wast:3345
assert_return(() => call($17, "load8_u", [20_496]), 0);

// memory_copy.wast:3346
assert_return(() => call($17, "load8_u", [20_695]), 0);

// memory_copy.wast:3347
assert_return(() => call($17, "load8_u", [20_894]), 0);

// memory_copy.wast:3348
assert_return(() => call($17, "load8_u", [21_093]), 0);

// memory_copy.wast:3349
assert_return(() => call($17, "load8_u", [21_292]), 0);

// memory_copy.wast:3350
assert_return(() => call($17, "load8_u", [21_491]), 0);

// memory_copy.wast:3351
assert_return(() => call($17, "load8_u", [21_690]), 0);

// memory_copy.wast:3352
assert_return(() => call($17, "load8_u", [21_889]), 0);

// memory_copy.wast:3353
assert_return(() => call($17, "load8_u", [22_088]), 0);

// memory_copy.wast:3354
assert_return(() => call($17, "load8_u", [22_287]), 0);

// memory_copy.wast:3355
assert_return(() => call($17, "load8_u", [22_486]), 0);

// memory_copy.wast:3356
assert_return(() => call($17, "load8_u", [22_685]), 0);

// memory_copy.wast:3357
assert_return(() => call($17, "load8_u", [22_884]), 0);

// memory_copy.wast:3358
assert_return(() => call($17, "load8_u", [23_083]), 0);

// memory_copy.wast:3359
assert_return(() => call($17, "load8_u", [23_282]), 0);

// memory_copy.wast:3360
assert_return(() => call($17, "load8_u", [23_481]), 0);

// memory_copy.wast:3361
assert_return(() => call($17, "load8_u", [23_680]), 0);

// memory_copy.wast:3362
assert_return(() => call($17, "load8_u", [23_879]), 0);

// memory_copy.wast:3363
assert_return(() => call($17, "load8_u", [24_078]), 0);

// memory_copy.wast:3364
assert_return(() => call($17, "load8_u", [24_277]), 0);

// memory_copy.wast:3365
assert_return(() => call($17, "load8_u", [24_476]), 0);

// memory_copy.wast:3366
assert_return(() => call($17, "load8_u", [24_675]), 0);

// memory_copy.wast:3367
assert_return(() => call($17, "load8_u", [24_874]), 0);

// memory_copy.wast:3368
assert_return(() => call($17, "load8_u", [25_073]), 0);

// memory_copy.wast:3369
assert_return(() => call($17, "load8_u", [25_272]), 0);

// memory_copy.wast:3370
assert_return(() => call($17, "load8_u", [25_471]), 0);

// memory_copy.wast:3371
assert_return(() => call($17, "load8_u", [25_670]), 0);

// memory_copy.wast:3372
assert_return(() => call($17, "load8_u", [25_869]), 0);

// memory_copy.wast:3373
assert_return(() => call($17, "load8_u", [26_068]), 0);

// memory_copy.wast:3374
assert_return(() => call($17, "load8_u", [26_267]), 0);

// memory_copy.wast:3375
assert_return(() => call($17, "load8_u", [26_466]), 0);

// memory_copy.wast:3376
assert_return(() => call($17, "load8_u", [26_665]), 0);

// memory_copy.wast:3377
assert_return(() => call($17, "load8_u", [26_864]), 0);

// memory_copy.wast:3378
assert_return(() => call($17, "load8_u", [27_063]), 0);

// memory_copy.wast:3379
assert_return(() => call($17, "load8_u", [27_262]), 0);

// memory_copy.wast:3380
assert_return(() => call($17, "load8_u", [27_461]), 0);

// memory_copy.wast:3381
assert_return(() => call($17, "load8_u", [27_660]), 0);

// memory_copy.wast:3382
assert_return(() => call($17, "load8_u", [27_859]), 0);

// memory_copy.wast:3383
assert_return(() => call($17, "load8_u", [28_058]), 0);

// memory_copy.wast:3384
assert_return(() => call($17, "load8_u", [28_257]), 0);

// memory_copy.wast:3385
assert_return(() => call($17, "load8_u", [28_456]), 0);

// memory_copy.wast:3386
assert_return(() => call($17, "load8_u", [28_655]), 0);

// memory_copy.wast:3387
assert_return(() => call($17, "load8_u", [28_854]), 0);

// memory_copy.wast:3388
assert_return(() => call($17, "load8_u", [29_053]), 0);

// memory_copy.wast:3389
assert_return(() => call($17, "load8_u", [29_252]), 0);

// memory_copy.wast:3390
assert_return(() => call($17, "load8_u", [29_451]), 0);

// memory_copy.wast:3391
assert_return(() => call($17, "load8_u", [29_650]), 0);

// memory_copy.wast:3392
assert_return(() => call($17, "load8_u", [29_849]), 0);

// memory_copy.wast:3393
assert_return(() => call($17, "load8_u", [30_048]), 0);

// memory_copy.wast:3394
assert_return(() => call($17, "load8_u", [30_247]), 0);

// memory_copy.wast:3395
assert_return(() => call($17, "load8_u", [30_446]), 0);

// memory_copy.wast:3396
assert_return(() => call($17, "load8_u", [30_645]), 0);

// memory_copy.wast:3397
assert_return(() => call($17, "load8_u", [30_844]), 0);

// memory_copy.wast:3398
assert_return(() => call($17, "load8_u", [31_043]), 0);

// memory_copy.wast:3399
assert_return(() => call($17, "load8_u", [31_242]), 0);

// memory_copy.wast:3400
assert_return(() => call($17, "load8_u", [31_441]), 0);

// memory_copy.wast:3401
assert_return(() => call($17, "load8_u", [31_640]), 0);

// memory_copy.wast:3402
assert_return(() => call($17, "load8_u", [31_839]), 0);

// memory_copy.wast:3403
assert_return(() => call($17, "load8_u", [32_038]), 0);

// memory_copy.wast:3404
assert_return(() => call($17, "load8_u", [32_237]), 0);

// memory_copy.wast:3405
assert_return(() => call($17, "load8_u", [32_436]), 0);

// memory_copy.wast:3406
assert_return(() => call($17, "load8_u", [32_635]), 0);

// memory_copy.wast:3407
assert_return(() => call($17, "load8_u", [32_834]), 0);

// memory_copy.wast:3408
assert_return(() => call($17, "load8_u", [33_033]), 0);

// memory_copy.wast:3409
assert_return(() => call($17, "load8_u", [33_232]), 0);

// memory_copy.wast:3410
assert_return(() => call($17, "load8_u", [33_431]), 0);

// memory_copy.wast:3411
assert_return(() => call($17, "load8_u", [33_630]), 0);

// memory_copy.wast:3412
assert_return(() => call($17, "load8_u", [33_829]), 0);

// memory_copy.wast:3413
assert_return(() => call($17, "load8_u", [34_028]), 0);

// memory_copy.wast:3414
assert_return(() => call($17, "load8_u", [34_227]), 0);

// memory_copy.wast:3415
assert_return(() => call($17, "load8_u", [34_426]), 0);

// memory_copy.wast:3416
assert_return(() => call($17, "load8_u", [34_625]), 0);

// memory_copy.wast:3417
assert_return(() => call($17, "load8_u", [34_824]), 0);

// memory_copy.wast:3418
assert_return(() => call($17, "load8_u", [35_023]), 0);

// memory_copy.wast:3419
assert_return(() => call($17, "load8_u", [35_222]), 0);

// memory_copy.wast:3420
assert_return(() => call($17, "load8_u", [35_421]), 0);

// memory_copy.wast:3421
assert_return(() => call($17, "load8_u", [35_620]), 0);

// memory_copy.wast:3422
assert_return(() => call($17, "load8_u", [35_819]), 0);

// memory_copy.wast:3423
assert_return(() => call($17, "load8_u", [36_018]), 0);

// memory_copy.wast:3424
assert_return(() => call($17, "load8_u", [36_217]), 0);

// memory_copy.wast:3425
assert_return(() => call($17, "load8_u", [36_416]), 0);

// memory_copy.wast:3426
assert_return(() => call($17, "load8_u", [36_615]), 0);

// memory_copy.wast:3427
assert_return(() => call($17, "load8_u", [36_814]), 0);

// memory_copy.wast:3428
assert_return(() => call($17, "load8_u", [37_013]), 0);

// memory_copy.wast:3429
assert_return(() => call($17, "load8_u", [37_212]), 0);

// memory_copy.wast:3430
assert_return(() => call($17, "load8_u", [37_411]), 0);

// memory_copy.wast:3431
assert_return(() => call($17, "load8_u", [37_610]), 0);

// memory_copy.wast:3432
assert_return(() => call($17, "load8_u", [37_809]), 0);

// memory_copy.wast:3433
assert_return(() => call($17, "load8_u", [38_008]), 0);

// memory_copy.wast:3434
assert_return(() => call($17, "load8_u", [38_207]), 0);

// memory_copy.wast:3435
assert_return(() => call($17, "load8_u", [38_406]), 0);

// memory_copy.wast:3436
assert_return(() => call($17, "load8_u", [38_605]), 0);

// memory_copy.wast:3437
assert_return(() => call($17, "load8_u", [38_804]), 0);

// memory_copy.wast:3438
assert_return(() => call($17, "load8_u", [39_003]), 0);

// memory_copy.wast:3439
assert_return(() => call($17, "load8_u", [39_202]), 0);

// memory_copy.wast:3440
assert_return(() => call($17, "load8_u", [39_401]), 0);

// memory_copy.wast:3441
assert_return(() => call($17, "load8_u", [39_600]), 0);

// memory_copy.wast:3442
assert_return(() => call($17, "load8_u", [39_799]), 0);

// memory_copy.wast:3443
assert_return(() => call($17, "load8_u", [39_998]), 0);

// memory_copy.wast:3444
assert_return(() => call($17, "load8_u", [40_197]), 0);

// memory_copy.wast:3445
assert_return(() => call($17, "load8_u", [40_396]), 0);

// memory_copy.wast:3446
assert_return(() => call($17, "load8_u", [40_595]), 0);

// memory_copy.wast:3447
assert_return(() => call($17, "load8_u", [40_794]), 0);

// memory_copy.wast:3448
assert_return(() => call($17, "load8_u", [40_993]), 0);

// memory_copy.wast:3449
assert_return(() => call($17, "load8_u", [41_192]), 0);

// memory_copy.wast:3450
assert_return(() => call($17, "load8_u", [41_391]), 0);

// memory_copy.wast:3451
assert_return(() => call($17, "load8_u", [41_590]), 0);

// memory_copy.wast:3452
assert_return(() => call($17, "load8_u", [41_789]), 0);

// memory_copy.wast:3453
assert_return(() => call($17, "load8_u", [41_988]), 0);

// memory_copy.wast:3454
assert_return(() => call($17, "load8_u", [42_187]), 0);

// memory_copy.wast:3455
assert_return(() => call($17, "load8_u", [42_386]), 0);

// memory_copy.wast:3456
assert_return(() => call($17, "load8_u", [42_585]), 0);

// memory_copy.wast:3457
assert_return(() => call($17, "load8_u", [42_784]), 0);

// memory_copy.wast:3458
assert_return(() => call($17, "load8_u", [42_983]), 0);

// memory_copy.wast:3459
assert_return(() => call($17, "load8_u", [43_182]), 0);

// memory_copy.wast:3460
assert_return(() => call($17, "load8_u", [43_381]), 0);

// memory_copy.wast:3461
assert_return(() => call($17, "load8_u", [43_580]), 0);

// memory_copy.wast:3462
assert_return(() => call($17, "load8_u", [43_779]), 0);

// memory_copy.wast:3463
assert_return(() => call($17, "load8_u", [43_978]), 0);

// memory_copy.wast:3464
assert_return(() => call($17, "load8_u", [44_177]), 0);

// memory_copy.wast:3465
assert_return(() => call($17, "load8_u", [44_376]), 0);

// memory_copy.wast:3466
assert_return(() => call($17, "load8_u", [44_575]), 0);

// memory_copy.wast:3467
assert_return(() => call($17, "load8_u", [44_774]), 0);

// memory_copy.wast:3468
assert_return(() => call($17, "load8_u", [44_973]), 0);

// memory_copy.wast:3469
assert_return(() => call($17, "load8_u", [45_172]), 0);

// memory_copy.wast:3470
assert_return(() => call($17, "load8_u", [45_371]), 0);

// memory_copy.wast:3471
assert_return(() => call($17, "load8_u", [45_570]), 0);

// memory_copy.wast:3472
assert_return(() => call($17, "load8_u", [45_769]), 0);

// memory_copy.wast:3473
assert_return(() => call($17, "load8_u", [45_968]), 0);

// memory_copy.wast:3474
assert_return(() => call($17, "load8_u", [46_167]), 0);

// memory_copy.wast:3475
assert_return(() => call($17, "load8_u", [46_366]), 0);

// memory_copy.wast:3476
assert_return(() => call($17, "load8_u", [46_565]), 0);

// memory_copy.wast:3477
assert_return(() => call($17, "load8_u", [46_764]), 0);

// memory_copy.wast:3478
assert_return(() => call($17, "load8_u", [46_963]), 0);

// memory_copy.wast:3479
assert_return(() => call($17, "load8_u", [47_162]), 0);

// memory_copy.wast:3480
assert_return(() => call($17, "load8_u", [47_361]), 0);

// memory_copy.wast:3481
assert_return(() => call($17, "load8_u", [47_560]), 0);

// memory_copy.wast:3482
assert_return(() => call($17, "load8_u", [47_759]), 0);

// memory_copy.wast:3483
assert_return(() => call($17, "load8_u", [47_958]), 0);

// memory_copy.wast:3484
assert_return(() => call($17, "load8_u", [48_157]), 0);

// memory_copy.wast:3485
assert_return(() => call($17, "load8_u", [48_356]), 0);

// memory_copy.wast:3486
assert_return(() => call($17, "load8_u", [48_555]), 0);

// memory_copy.wast:3487
assert_return(() => call($17, "load8_u", [48_754]), 0);

// memory_copy.wast:3488
assert_return(() => call($17, "load8_u", [48_953]), 0);

// memory_copy.wast:3489
assert_return(() => call($17, "load8_u", [49_152]), 0);

// memory_copy.wast:3490
assert_return(() => call($17, "load8_u", [49_351]), 0);

// memory_copy.wast:3491
assert_return(() => call($17, "load8_u", [49_550]), 0);

// memory_copy.wast:3492
assert_return(() => call($17, "load8_u", [49_749]), 0);

// memory_copy.wast:3493
assert_return(() => call($17, "load8_u", [49_948]), 0);

// memory_copy.wast:3494
assert_return(() => call($17, "load8_u", [50_147]), 0);

// memory_copy.wast:3495
assert_return(() => call($17, "load8_u", [50_346]), 0);

// memory_copy.wast:3496
assert_return(() => call($17, "load8_u", [50_545]), 0);

// memory_copy.wast:3497
assert_return(() => call($17, "load8_u", [50_744]), 0);

// memory_copy.wast:3498
assert_return(() => call($17, "load8_u", [50_943]), 0);

// memory_copy.wast:3499
assert_return(() => call($17, "load8_u", [51_142]), 0);

// memory_copy.wast:3500
assert_return(() => call($17, "load8_u", [51_341]), 0);

// memory_copy.wast:3501
assert_return(() => call($17, "load8_u", [51_540]), 0);

// memory_copy.wast:3502
assert_return(() => call($17, "load8_u", [51_739]), 0);

// memory_copy.wast:3503
assert_return(() => call($17, "load8_u", [51_938]), 0);

// memory_copy.wast:3504
assert_return(() => call($17, "load8_u", [52_137]), 0);

// memory_copy.wast:3505
assert_return(() => call($17, "load8_u", [52_336]), 0);

// memory_copy.wast:3506
assert_return(() => call($17, "load8_u", [52_535]), 0);

// memory_copy.wast:3507
assert_return(() => call($17, "load8_u", [52_734]), 0);

// memory_copy.wast:3508
assert_return(() => call($17, "load8_u", [52_933]), 0);

// memory_copy.wast:3509
assert_return(() => call($17, "load8_u", [53_132]), 0);

// memory_copy.wast:3510
assert_return(() => call($17, "load8_u", [53_331]), 0);

// memory_copy.wast:3511
assert_return(() => call($17, "load8_u", [53_530]), 0);

// memory_copy.wast:3512
assert_return(() => call($17, "load8_u", [53_729]), 0);

// memory_copy.wast:3513
assert_return(() => call($17, "load8_u", [53_928]), 0);

// memory_copy.wast:3514
assert_return(() => call($17, "load8_u", [54_127]), 0);

// memory_copy.wast:3515
assert_return(() => call($17, "load8_u", [54_326]), 0);

// memory_copy.wast:3516
assert_return(() => call($17, "load8_u", [54_525]), 0);

// memory_copy.wast:3517
assert_return(() => call($17, "load8_u", [54_724]), 0);

// memory_copy.wast:3518
assert_return(() => call($17, "load8_u", [54_923]), 0);

// memory_copy.wast:3519
assert_return(() => call($17, "load8_u", [55_122]), 0);

// memory_copy.wast:3520
assert_return(() => call($17, "load8_u", [55_321]), 0);

// memory_copy.wast:3521
assert_return(() => call($17, "load8_u", [55_520]), 0);

// memory_copy.wast:3522
assert_return(() => call($17, "load8_u", [55_719]), 0);

// memory_copy.wast:3523
assert_return(() => call($17, "load8_u", [55_918]), 0);

// memory_copy.wast:3524
assert_return(() => call($17, "load8_u", [56_117]), 0);

// memory_copy.wast:3525
assert_return(() => call($17, "load8_u", [56_316]), 0);

// memory_copy.wast:3526
assert_return(() => call($17, "load8_u", [56_515]), 0);

// memory_copy.wast:3527
assert_return(() => call($17, "load8_u", [56_714]), 0);

// memory_copy.wast:3528
assert_return(() => call($17, "load8_u", [56_913]), 0);

// memory_copy.wast:3529
assert_return(() => call($17, "load8_u", [57_112]), 0);

// memory_copy.wast:3530
assert_return(() => call($17, "load8_u", [57_311]), 0);

// memory_copy.wast:3531
assert_return(() => call($17, "load8_u", [57_510]), 0);

// memory_copy.wast:3532
assert_return(() => call($17, "load8_u", [57_709]), 0);

// memory_copy.wast:3533
assert_return(() => call($17, "load8_u", [57_908]), 0);

// memory_copy.wast:3534
assert_return(() => call($17, "load8_u", [58_107]), 0);

// memory_copy.wast:3535
assert_return(() => call($17, "load8_u", [58_306]), 0);

// memory_copy.wast:3536
assert_return(() => call($17, "load8_u", [58_505]), 0);

// memory_copy.wast:3537
assert_return(() => call($17, "load8_u", [58_704]), 0);

// memory_copy.wast:3538
assert_return(() => call($17, "load8_u", [58_903]), 0);

// memory_copy.wast:3539
assert_return(() => call($17, "load8_u", [59_102]), 0);

// memory_copy.wast:3540
assert_return(() => call($17, "load8_u", [59_301]), 0);

// memory_copy.wast:3541
assert_return(() => call($17, "load8_u", [59_500]), 0);

// memory_copy.wast:3542
assert_return(() => call($17, "load8_u", [59_699]), 0);

// memory_copy.wast:3543
assert_return(() => call($17, "load8_u", [59_898]), 0);

// memory_copy.wast:3544
assert_return(() => call($17, "load8_u", [60_097]), 0);

// memory_copy.wast:3545
assert_return(() => call($17, "load8_u", [60_296]), 0);

// memory_copy.wast:3546
assert_return(() => call($17, "load8_u", [60_495]), 0);

// memory_copy.wast:3547
assert_return(() => call($17, "load8_u", [60_694]), 0);

// memory_copy.wast:3548
assert_return(() => call($17, "load8_u", [60_893]), 0);

// memory_copy.wast:3549
assert_return(() => call($17, "load8_u", [61_092]), 0);

// memory_copy.wast:3550
assert_return(() => call($17, "load8_u", [61_291]), 0);

// memory_copy.wast:3551
assert_return(() => call($17, "load8_u", [61_490]), 0);

// memory_copy.wast:3552
assert_return(() => call($17, "load8_u", [61_689]), 0);

// memory_copy.wast:3553
assert_return(() => call($17, "load8_u", [61_888]), 0);

// memory_copy.wast:3554
assert_return(() => call($17, "load8_u", [62_087]), 0);

// memory_copy.wast:3555
assert_return(() => call($17, "load8_u", [62_286]), 0);

// memory_copy.wast:3556
assert_return(() => call($17, "load8_u", [62_485]), 0);

// memory_copy.wast:3557
assert_return(() => call($17, "load8_u", [62_684]), 0);

// memory_copy.wast:3558
assert_return(() => call($17, "load8_u", [62_883]), 0);

// memory_copy.wast:3559
assert_return(() => call($17, "load8_u", [63_082]), 0);

// memory_copy.wast:3560
assert_return(() => call($17, "load8_u", [63_281]), 0);

// memory_copy.wast:3561
assert_return(() => call($17, "load8_u", [63_480]), 0);

// memory_copy.wast:3562
assert_return(() => call($17, "load8_u", [63_679]), 0);

// memory_copy.wast:3563
assert_return(() => call($17, "load8_u", [63_878]), 0);

// memory_copy.wast:3564
assert_return(() => call($17, "load8_u", [64_077]), 0);

// memory_copy.wast:3565
assert_return(() => call($17, "load8_u", [64_276]), 0);

// memory_copy.wast:3566
assert_return(() => call($17, "load8_u", [64_475]), 0);

// memory_copy.wast:3567
assert_return(() => call($17, "load8_u", [64_674]), 0);

// memory_copy.wast:3568
assert_return(() => call($17, "load8_u", [64_873]), 0);

// memory_copy.wast:3569
assert_return(() => call($17, "load8_u", [65_072]), 0);

// memory_copy.wast:3570
assert_return(() => call($17, "load8_u", [65_271]), 0);

// memory_copy.wast:3571
assert_return(() => call($17, "load8_u", [65_470]), 0);

// memory_copy.wast:3572
assert_return(() => call($17, "load8_u", [65_516]), 0);

// memory_copy.wast:3573
assert_return(() => call($17, "load8_u", [65_517]), 1);

// memory_copy.wast:3574
assert_return(() => call($17, "load8_u", [65_518]), 2);

// memory_copy.wast:3575
assert_return(() => call($17, "load8_u", [65_519]), 3);

// memory_copy.wast:3576
assert_return(() => call($17, "load8_u", [65_520]), 4);

// memory_copy.wast:3577
assert_return(() => call($17, "load8_u", [65_521]), 5);

// memory_copy.wast:3578
assert_return(() => call($17, "load8_u", [65_522]), 6);

// memory_copy.wast:3579
assert_return(() => call($17, "load8_u", [65_523]), 7);

// memory_copy.wast:3580
assert_return(() => call($17, "load8_u", [65_524]), 8);

// memory_copy.wast:3581
assert_return(() => call($17, "load8_u", [65_525]), 9);

// memory_copy.wast:3582
assert_return(() => call($17, "load8_u", [65_526]), 10);

// memory_copy.wast:3583
assert_return(() => call($17, "load8_u", [65_527]), 11);

// memory_copy.wast:3584
assert_return(() => call($17, "load8_u", [65_528]), 12);

// memory_copy.wast:3585
assert_return(() => call($17, "load8_u", [65_529]), 13);

// memory_copy.wast:3586
assert_return(() => call($17, "load8_u", [65_530]), 14);

// memory_copy.wast:3587
assert_return(() => call($17, "load8_u", [65_531]), 15);

// memory_copy.wast:3588
assert_return(() => call($17, "load8_u", [65_532]), 16);

// memory_copy.wast:3589
assert_return(() => call($17, "load8_u", [65_533]), 17);

// memory_copy.wast:3590
assert_return(() => call($17, "load8_u", [65_534]), 18);

// memory_copy.wast:3591
assert_return(() => call($17, "load8_u", [65_535]), 19);

// memory_copy.wast:3593
let $18 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x83\x80\x80\x80\x00\x01\x00\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9c\x80\x80\x80\x00\x01\x00\x41\xec\xff\x03\x0b\x14\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

// memory_copy.wast:3601
assert_trap(() => call($18, "run", [0, 65_516, -4_096]));

// memory_copy.wast:3604
assert_return(() => call($18, "load8_u", [198]), 0);

// memory_copy.wast:3605
assert_return(() => call($18, "load8_u", [397]), 0);

// memory_copy.wast:3606
assert_return(() => call($18, "load8_u", [596]), 0);

// memory_copy.wast:3607
assert_return(() => call($18, "load8_u", [795]), 0);

// memory_copy.wast:3608
assert_return(() => call($18, "load8_u", [994]), 0);

// memory_copy.wast:3609
assert_return(() => call($18, "load8_u", [1_193]), 0);

// memory_copy.wast:3610
assert_return(() => call($18, "load8_u", [1_392]), 0);

// memory_copy.wast:3611
assert_return(() => call($18, "load8_u", [1_591]), 0);

// memory_copy.wast:3612
assert_return(() => call($18, "load8_u", [1_790]), 0);

// memory_copy.wast:3613
assert_return(() => call($18, "load8_u", [1_989]), 0);

// memory_copy.wast:3614
assert_return(() => call($18, "load8_u", [2_188]), 0);

// memory_copy.wast:3615
assert_return(() => call($18, "load8_u", [2_387]), 0);

// memory_copy.wast:3616
assert_return(() => call($18, "load8_u", [2_586]), 0);

// memory_copy.wast:3617
assert_return(() => call($18, "load8_u", [2_785]), 0);

// memory_copy.wast:3618
assert_return(() => call($18, "load8_u", [2_984]), 0);

// memory_copy.wast:3619
assert_return(() => call($18, "load8_u", [3_183]), 0);

// memory_copy.wast:3620
assert_return(() => call($18, "load8_u", [3_382]), 0);

// memory_copy.wast:3621
assert_return(() => call($18, "load8_u", [3_581]), 0);

// memory_copy.wast:3622
assert_return(() => call($18, "load8_u", [3_780]), 0);

// memory_copy.wast:3623
assert_return(() => call($18, "load8_u", [3_979]), 0);

// memory_copy.wast:3624
assert_return(() => call($18, "load8_u", [4_178]), 0);

// memory_copy.wast:3625
assert_return(() => call($18, "load8_u", [4_377]), 0);

// memory_copy.wast:3626
assert_return(() => call($18, "load8_u", [4_576]), 0);

// memory_copy.wast:3627
assert_return(() => call($18, "load8_u", [4_775]), 0);

// memory_copy.wast:3628
assert_return(() => call($18, "load8_u", [4_974]), 0);

// memory_copy.wast:3629
assert_return(() => call($18, "load8_u", [5_173]), 0);

// memory_copy.wast:3630
assert_return(() => call($18, "load8_u", [5_372]), 0);

// memory_copy.wast:3631
assert_return(() => call($18, "load8_u", [5_571]), 0);

// memory_copy.wast:3632
assert_return(() => call($18, "load8_u", [5_770]), 0);

// memory_copy.wast:3633
assert_return(() => call($18, "load8_u", [5_969]), 0);

// memory_copy.wast:3634
assert_return(() => call($18, "load8_u", [6_168]), 0);

// memory_copy.wast:3635
assert_return(() => call($18, "load8_u", [6_367]), 0);

// memory_copy.wast:3636
assert_return(() => call($18, "load8_u", [6_566]), 0);

// memory_copy.wast:3637
assert_return(() => call($18, "load8_u", [6_765]), 0);

// memory_copy.wast:3638
assert_return(() => call($18, "load8_u", [6_964]), 0);

// memory_copy.wast:3639
assert_return(() => call($18, "load8_u", [7_163]), 0);

// memory_copy.wast:3640
assert_return(() => call($18, "load8_u", [7_362]), 0);

// memory_copy.wast:3641
assert_return(() => call($18, "load8_u", [7_561]), 0);

// memory_copy.wast:3642
assert_return(() => call($18, "load8_u", [7_760]), 0);

// memory_copy.wast:3643
assert_return(() => call($18, "load8_u", [7_959]), 0);

// memory_copy.wast:3644
assert_return(() => call($18, "load8_u", [8_158]), 0);

// memory_copy.wast:3645
assert_return(() => call($18, "load8_u", [8_357]), 0);

// memory_copy.wast:3646
assert_return(() => call($18, "load8_u", [8_556]), 0);

// memory_copy.wast:3647
assert_return(() => call($18, "load8_u", [8_755]), 0);

// memory_copy.wast:3648
assert_return(() => call($18, "load8_u", [8_954]), 0);

// memory_copy.wast:3649
assert_return(() => call($18, "load8_u", [9_153]), 0);

// memory_copy.wast:3650
assert_return(() => call($18, "load8_u", [9_352]), 0);

// memory_copy.wast:3651
assert_return(() => call($18, "load8_u", [9_551]), 0);

// memory_copy.wast:3652
assert_return(() => call($18, "load8_u", [9_750]), 0);

// memory_copy.wast:3653
assert_return(() => call($18, "load8_u", [9_949]), 0);

// memory_copy.wast:3654
assert_return(() => call($18, "load8_u", [10_148]), 0);

// memory_copy.wast:3655
assert_return(() => call($18, "load8_u", [10_347]), 0);

// memory_copy.wast:3656
assert_return(() => call($18, "load8_u", [10_546]), 0);

// memory_copy.wast:3657
assert_return(() => call($18, "load8_u", [10_745]), 0);

// memory_copy.wast:3658
assert_return(() => call($18, "load8_u", [10_944]), 0);

// memory_copy.wast:3659
assert_return(() => call($18, "load8_u", [11_143]), 0);

// memory_copy.wast:3660
assert_return(() => call($18, "load8_u", [11_342]), 0);

// memory_copy.wast:3661
assert_return(() => call($18, "load8_u", [11_541]), 0);

// memory_copy.wast:3662
assert_return(() => call($18, "load8_u", [11_740]), 0);

// memory_copy.wast:3663
assert_return(() => call($18, "load8_u", [11_939]), 0);

// memory_copy.wast:3664
assert_return(() => call($18, "load8_u", [12_138]), 0);

// memory_copy.wast:3665
assert_return(() => call($18, "load8_u", [12_337]), 0);

// memory_copy.wast:3666
assert_return(() => call($18, "load8_u", [12_536]), 0);

// memory_copy.wast:3667
assert_return(() => call($18, "load8_u", [12_735]), 0);

// memory_copy.wast:3668
assert_return(() => call($18, "load8_u", [12_934]), 0);

// memory_copy.wast:3669
assert_return(() => call($18, "load8_u", [13_133]), 0);

// memory_copy.wast:3670
assert_return(() => call($18, "load8_u", [13_332]), 0);

// memory_copy.wast:3671
assert_return(() => call($18, "load8_u", [13_531]), 0);

// memory_copy.wast:3672
assert_return(() => call($18, "load8_u", [13_730]), 0);

// memory_copy.wast:3673
assert_return(() => call($18, "load8_u", [13_929]), 0);

// memory_copy.wast:3674
assert_return(() => call($18, "load8_u", [14_128]), 0);

// memory_copy.wast:3675
assert_return(() => call($18, "load8_u", [14_327]), 0);

// memory_copy.wast:3676
assert_return(() => call($18, "load8_u", [14_526]), 0);

// memory_copy.wast:3677
assert_return(() => call($18, "load8_u", [14_725]), 0);

// memory_copy.wast:3678
assert_return(() => call($18, "load8_u", [14_924]), 0);

// memory_copy.wast:3679
assert_return(() => call($18, "load8_u", [15_123]), 0);

// memory_copy.wast:3680
assert_return(() => call($18, "load8_u", [15_322]), 0);

// memory_copy.wast:3681
assert_return(() => call($18, "load8_u", [15_521]), 0);

// memory_copy.wast:3682
assert_return(() => call($18, "load8_u", [15_720]), 0);

// memory_copy.wast:3683
assert_return(() => call($18, "load8_u", [15_919]), 0);

// memory_copy.wast:3684
assert_return(() => call($18, "load8_u", [16_118]), 0);

// memory_copy.wast:3685
assert_return(() => call($18, "load8_u", [16_317]), 0);

// memory_copy.wast:3686
assert_return(() => call($18, "load8_u", [16_516]), 0);

// memory_copy.wast:3687
assert_return(() => call($18, "load8_u", [16_715]), 0);

// memory_copy.wast:3688
assert_return(() => call($18, "load8_u", [16_914]), 0);

// memory_copy.wast:3689
assert_return(() => call($18, "load8_u", [17_113]), 0);

// memory_copy.wast:3690
assert_return(() => call($18, "load8_u", [17_312]), 0);

// memory_copy.wast:3691
assert_return(() => call($18, "load8_u", [17_511]), 0);

// memory_copy.wast:3692
assert_return(() => call($18, "load8_u", [17_710]), 0);

// memory_copy.wast:3693
assert_return(() => call($18, "load8_u", [17_909]), 0);

// memory_copy.wast:3694
assert_return(() => call($18, "load8_u", [18_108]), 0);

// memory_copy.wast:3695
assert_return(() => call($18, "load8_u", [18_307]), 0);

// memory_copy.wast:3696
assert_return(() => call($18, "load8_u", [18_506]), 0);

// memory_copy.wast:3697
assert_return(() => call($18, "load8_u", [18_705]), 0);

// memory_copy.wast:3698
assert_return(() => call($18, "load8_u", [18_904]), 0);

// memory_copy.wast:3699
assert_return(() => call($18, "load8_u", [19_103]), 0);

// memory_copy.wast:3700
assert_return(() => call($18, "load8_u", [19_302]), 0);

// memory_copy.wast:3701
assert_return(() => call($18, "load8_u", [19_501]), 0);

// memory_copy.wast:3702
assert_return(() => call($18, "load8_u", [19_700]), 0);

// memory_copy.wast:3703
assert_return(() => call($18, "load8_u", [19_899]), 0);

// memory_copy.wast:3704
assert_return(() => call($18, "load8_u", [20_098]), 0);

// memory_copy.wast:3705
assert_return(() => call($18, "load8_u", [20_297]), 0);

// memory_copy.wast:3706
assert_return(() => call($18, "load8_u", [20_496]), 0);

// memory_copy.wast:3707
assert_return(() => call($18, "load8_u", [20_695]), 0);

// memory_copy.wast:3708
assert_return(() => call($18, "load8_u", [20_894]), 0);

// memory_copy.wast:3709
assert_return(() => call($18, "load8_u", [21_093]), 0);

// memory_copy.wast:3710
assert_return(() => call($18, "load8_u", [21_292]), 0);

// memory_copy.wast:3711
assert_return(() => call($18, "load8_u", [21_491]), 0);

// memory_copy.wast:3712
assert_return(() => call($18, "load8_u", [21_690]), 0);

// memory_copy.wast:3713
assert_return(() => call($18, "load8_u", [21_889]), 0);

// memory_copy.wast:3714
assert_return(() => call($18, "load8_u", [22_088]), 0);

// memory_copy.wast:3715
assert_return(() => call($18, "load8_u", [22_287]), 0);

// memory_copy.wast:3716
assert_return(() => call($18, "load8_u", [22_486]), 0);

// memory_copy.wast:3717
assert_return(() => call($18, "load8_u", [22_685]), 0);

// memory_copy.wast:3718
assert_return(() => call($18, "load8_u", [22_884]), 0);

// memory_copy.wast:3719
assert_return(() => call($18, "load8_u", [23_083]), 0);

// memory_copy.wast:3720
assert_return(() => call($18, "load8_u", [23_282]), 0);

// memory_copy.wast:3721
assert_return(() => call($18, "load8_u", [23_481]), 0);

// memory_copy.wast:3722
assert_return(() => call($18, "load8_u", [23_680]), 0);

// memory_copy.wast:3723
assert_return(() => call($18, "load8_u", [23_879]), 0);

// memory_copy.wast:3724
assert_return(() => call($18, "load8_u", [24_078]), 0);

// memory_copy.wast:3725
assert_return(() => call($18, "load8_u", [24_277]), 0);

// memory_copy.wast:3726
assert_return(() => call($18, "load8_u", [24_476]), 0);

// memory_copy.wast:3727
assert_return(() => call($18, "load8_u", [24_675]), 0);

// memory_copy.wast:3728
assert_return(() => call($18, "load8_u", [24_874]), 0);

// memory_copy.wast:3729
assert_return(() => call($18, "load8_u", [25_073]), 0);

// memory_copy.wast:3730
assert_return(() => call($18, "load8_u", [25_272]), 0);

// memory_copy.wast:3731
assert_return(() => call($18, "load8_u", [25_471]), 0);

// memory_copy.wast:3732
assert_return(() => call($18, "load8_u", [25_670]), 0);

// memory_copy.wast:3733
assert_return(() => call($18, "load8_u", [25_869]), 0);

// memory_copy.wast:3734
assert_return(() => call($18, "load8_u", [26_068]), 0);

// memory_copy.wast:3735
assert_return(() => call($18, "load8_u", [26_267]), 0);

// memory_copy.wast:3736
assert_return(() => call($18, "load8_u", [26_466]), 0);

// memory_copy.wast:3737
assert_return(() => call($18, "load8_u", [26_665]), 0);

// memory_copy.wast:3738
assert_return(() => call($18, "load8_u", [26_864]), 0);

// memory_copy.wast:3739
assert_return(() => call($18, "load8_u", [27_063]), 0);

// memory_copy.wast:3740
assert_return(() => call($18, "load8_u", [27_262]), 0);

// memory_copy.wast:3741
assert_return(() => call($18, "load8_u", [27_461]), 0);

// memory_copy.wast:3742
assert_return(() => call($18, "load8_u", [27_660]), 0);

// memory_copy.wast:3743
assert_return(() => call($18, "load8_u", [27_859]), 0);

// memory_copy.wast:3744
assert_return(() => call($18, "load8_u", [28_058]), 0);

// memory_copy.wast:3745
assert_return(() => call($18, "load8_u", [28_257]), 0);

// memory_copy.wast:3746
assert_return(() => call($18, "load8_u", [28_456]), 0);

// memory_copy.wast:3747
assert_return(() => call($18, "load8_u", [28_655]), 0);

// memory_copy.wast:3748
assert_return(() => call($18, "load8_u", [28_854]), 0);

// memory_copy.wast:3749
assert_return(() => call($18, "load8_u", [29_053]), 0);

// memory_copy.wast:3750
assert_return(() => call($18, "load8_u", [29_252]), 0);

// memory_copy.wast:3751
assert_return(() => call($18, "load8_u", [29_451]), 0);

// memory_copy.wast:3752
assert_return(() => call($18, "load8_u", [29_650]), 0);

// memory_copy.wast:3753
assert_return(() => call($18, "load8_u", [29_849]), 0);

// memory_copy.wast:3754
assert_return(() => call($18, "load8_u", [30_048]), 0);

// memory_copy.wast:3755
assert_return(() => call($18, "load8_u", [30_247]), 0);

// memory_copy.wast:3756
assert_return(() => call($18, "load8_u", [30_446]), 0);

// memory_copy.wast:3757
assert_return(() => call($18, "load8_u", [30_645]), 0);

// memory_copy.wast:3758
assert_return(() => call($18, "load8_u", [30_844]), 0);

// memory_copy.wast:3759
assert_return(() => call($18, "load8_u", [31_043]), 0);

// memory_copy.wast:3760
assert_return(() => call($18, "load8_u", [31_242]), 0);

// memory_copy.wast:3761
assert_return(() => call($18, "load8_u", [31_441]), 0);

// memory_copy.wast:3762
assert_return(() => call($18, "load8_u", [31_640]), 0);

// memory_copy.wast:3763
assert_return(() => call($18, "load8_u", [31_839]), 0);

// memory_copy.wast:3764
assert_return(() => call($18, "load8_u", [32_038]), 0);

// memory_copy.wast:3765
assert_return(() => call($18, "load8_u", [32_237]), 0);

// memory_copy.wast:3766
assert_return(() => call($18, "load8_u", [32_436]), 0);

// memory_copy.wast:3767
assert_return(() => call($18, "load8_u", [32_635]), 0);

// memory_copy.wast:3768
assert_return(() => call($18, "load8_u", [32_834]), 0);

// memory_copy.wast:3769
assert_return(() => call($18, "load8_u", [33_033]), 0);

// memory_copy.wast:3770
assert_return(() => call($18, "load8_u", [33_232]), 0);

// memory_copy.wast:3771
assert_return(() => call($18, "load8_u", [33_431]), 0);

// memory_copy.wast:3772
assert_return(() => call($18, "load8_u", [33_630]), 0);

// memory_copy.wast:3773
assert_return(() => call($18, "load8_u", [33_829]), 0);

// memory_copy.wast:3774
assert_return(() => call($18, "load8_u", [34_028]), 0);

// memory_copy.wast:3775
assert_return(() => call($18, "load8_u", [34_227]), 0);

// memory_copy.wast:3776
assert_return(() => call($18, "load8_u", [34_426]), 0);

// memory_copy.wast:3777
assert_return(() => call($18, "load8_u", [34_625]), 0);

// memory_copy.wast:3778
assert_return(() => call($18, "load8_u", [34_824]), 0);

// memory_copy.wast:3779
assert_return(() => call($18, "load8_u", [35_023]), 0);

// memory_copy.wast:3780
assert_return(() => call($18, "load8_u", [35_222]), 0);

// memory_copy.wast:3781
assert_return(() => call($18, "load8_u", [35_421]), 0);

// memory_copy.wast:3782
assert_return(() => call($18, "load8_u", [35_620]), 0);

// memory_copy.wast:3783
assert_return(() => call($18, "load8_u", [35_819]), 0);

// memory_copy.wast:3784
assert_return(() => call($18, "load8_u", [36_018]), 0);

// memory_copy.wast:3785
assert_return(() => call($18, "load8_u", [36_217]), 0);

// memory_copy.wast:3786
assert_return(() => call($18, "load8_u", [36_416]), 0);

// memory_copy.wast:3787
assert_return(() => call($18, "load8_u", [36_615]), 0);

// memory_copy.wast:3788
assert_return(() => call($18, "load8_u", [36_814]), 0);

// memory_copy.wast:3789
assert_return(() => call($18, "load8_u", [37_013]), 0);

// memory_copy.wast:3790
assert_return(() => call($18, "load8_u", [37_212]), 0);

// memory_copy.wast:3791
assert_return(() => call($18, "load8_u", [37_411]), 0);

// memory_copy.wast:3792
assert_return(() => call($18, "load8_u", [37_610]), 0);

// memory_copy.wast:3793
assert_return(() => call($18, "load8_u", [37_809]), 0);

// memory_copy.wast:3794
assert_return(() => call($18, "load8_u", [38_008]), 0);

// memory_copy.wast:3795
assert_return(() => call($18, "load8_u", [38_207]), 0);

// memory_copy.wast:3796
assert_return(() => call($18, "load8_u", [38_406]), 0);

// memory_copy.wast:3797
assert_return(() => call($18, "load8_u", [38_605]), 0);

// memory_copy.wast:3798
assert_return(() => call($18, "load8_u", [38_804]), 0);

// memory_copy.wast:3799
assert_return(() => call($18, "load8_u", [39_003]), 0);

// memory_copy.wast:3800
assert_return(() => call($18, "load8_u", [39_202]), 0);

// memory_copy.wast:3801
assert_return(() => call($18, "load8_u", [39_401]), 0);

// memory_copy.wast:3802
assert_return(() => call($18, "load8_u", [39_600]), 0);

// memory_copy.wast:3803
assert_return(() => call($18, "load8_u", [39_799]), 0);

// memory_copy.wast:3804
assert_return(() => call($18, "load8_u", [39_998]), 0);

// memory_copy.wast:3805
assert_return(() => call($18, "load8_u", [40_197]), 0);

// memory_copy.wast:3806
assert_return(() => call($18, "load8_u", [40_396]), 0);

// memory_copy.wast:3807
assert_return(() => call($18, "load8_u", [40_595]), 0);

// memory_copy.wast:3808
assert_return(() => call($18, "load8_u", [40_794]), 0);

// memory_copy.wast:3809
assert_return(() => call($18, "load8_u", [40_993]), 0);

// memory_copy.wast:3810
assert_return(() => call($18, "load8_u", [41_192]), 0);

// memory_copy.wast:3811
assert_return(() => call($18, "load8_u", [41_391]), 0);

// memory_copy.wast:3812
assert_return(() => call($18, "load8_u", [41_590]), 0);

// memory_copy.wast:3813
assert_return(() => call($18, "load8_u", [41_789]), 0);

// memory_copy.wast:3814
assert_return(() => call($18, "load8_u", [41_988]), 0);

// memory_copy.wast:3815
assert_return(() => call($18, "load8_u", [42_187]), 0);

// memory_copy.wast:3816
assert_return(() => call($18, "load8_u", [42_386]), 0);

// memory_copy.wast:3817
assert_return(() => call($18, "load8_u", [42_585]), 0);

// memory_copy.wast:3818
assert_return(() => call($18, "load8_u", [42_784]), 0);

// memory_copy.wast:3819
assert_return(() => call($18, "load8_u", [42_983]), 0);

// memory_copy.wast:3820
assert_return(() => call($18, "load8_u", [43_182]), 0);

// memory_copy.wast:3821
assert_return(() => call($18, "load8_u", [43_381]), 0);

// memory_copy.wast:3822
assert_return(() => call($18, "load8_u", [43_580]), 0);

// memory_copy.wast:3823
assert_return(() => call($18, "load8_u", [43_779]), 0);

// memory_copy.wast:3824
assert_return(() => call($18, "load8_u", [43_978]), 0);

// memory_copy.wast:3825
assert_return(() => call($18, "load8_u", [44_177]), 0);

// memory_copy.wast:3826
assert_return(() => call($18, "load8_u", [44_376]), 0);

// memory_copy.wast:3827
assert_return(() => call($18, "load8_u", [44_575]), 0);

// memory_copy.wast:3828
assert_return(() => call($18, "load8_u", [44_774]), 0);

// memory_copy.wast:3829
assert_return(() => call($18, "load8_u", [44_973]), 0);

// memory_copy.wast:3830
assert_return(() => call($18, "load8_u", [45_172]), 0);

// memory_copy.wast:3831
assert_return(() => call($18, "load8_u", [45_371]), 0);

// memory_copy.wast:3832
assert_return(() => call($18, "load8_u", [45_570]), 0);

// memory_copy.wast:3833
assert_return(() => call($18, "load8_u", [45_769]), 0);

// memory_copy.wast:3834
assert_return(() => call($18, "load8_u", [45_968]), 0);

// memory_copy.wast:3835
assert_return(() => call($18, "load8_u", [46_167]), 0);

// memory_copy.wast:3836
assert_return(() => call($18, "load8_u", [46_366]), 0);

// memory_copy.wast:3837
assert_return(() => call($18, "load8_u", [46_565]), 0);

// memory_copy.wast:3838
assert_return(() => call($18, "load8_u", [46_764]), 0);

// memory_copy.wast:3839
assert_return(() => call($18, "load8_u", [46_963]), 0);

// memory_copy.wast:3840
assert_return(() => call($18, "load8_u", [47_162]), 0);

// memory_copy.wast:3841
assert_return(() => call($18, "load8_u", [47_361]), 0);

// memory_copy.wast:3842
assert_return(() => call($18, "load8_u", [47_560]), 0);

// memory_copy.wast:3843
assert_return(() => call($18, "load8_u", [47_759]), 0);

// memory_copy.wast:3844
assert_return(() => call($18, "load8_u", [47_958]), 0);

// memory_copy.wast:3845
assert_return(() => call($18, "load8_u", [48_157]), 0);

// memory_copy.wast:3846
assert_return(() => call($18, "load8_u", [48_356]), 0);

// memory_copy.wast:3847
assert_return(() => call($18, "load8_u", [48_555]), 0);

// memory_copy.wast:3848
assert_return(() => call($18, "load8_u", [48_754]), 0);

// memory_copy.wast:3849
assert_return(() => call($18, "load8_u", [48_953]), 0);

// memory_copy.wast:3850
assert_return(() => call($18, "load8_u", [49_152]), 0);

// memory_copy.wast:3851
assert_return(() => call($18, "load8_u", [49_351]), 0);

// memory_copy.wast:3852
assert_return(() => call($18, "load8_u", [49_550]), 0);

// memory_copy.wast:3853
assert_return(() => call($18, "load8_u", [49_749]), 0);

// memory_copy.wast:3854
assert_return(() => call($18, "load8_u", [49_948]), 0);

// memory_copy.wast:3855
assert_return(() => call($18, "load8_u", [50_147]), 0);

// memory_copy.wast:3856
assert_return(() => call($18, "load8_u", [50_346]), 0);

// memory_copy.wast:3857
assert_return(() => call($18, "load8_u", [50_545]), 0);

// memory_copy.wast:3858
assert_return(() => call($18, "load8_u", [50_744]), 0);

// memory_copy.wast:3859
assert_return(() => call($18, "load8_u", [50_943]), 0);

// memory_copy.wast:3860
assert_return(() => call($18, "load8_u", [51_142]), 0);

// memory_copy.wast:3861
assert_return(() => call($18, "load8_u", [51_341]), 0);

// memory_copy.wast:3862
assert_return(() => call($18, "load8_u", [51_540]), 0);

// memory_copy.wast:3863
assert_return(() => call($18, "load8_u", [51_739]), 0);

// memory_copy.wast:3864
assert_return(() => call($18, "load8_u", [51_938]), 0);

// memory_copy.wast:3865
assert_return(() => call($18, "load8_u", [52_137]), 0);

// memory_copy.wast:3866
assert_return(() => call($18, "load8_u", [52_336]), 0);

// memory_copy.wast:3867
assert_return(() => call($18, "load8_u", [52_535]), 0);

// memory_copy.wast:3868
assert_return(() => call($18, "load8_u", [52_734]), 0);

// memory_copy.wast:3869
assert_return(() => call($18, "load8_u", [52_933]), 0);

// memory_copy.wast:3870
assert_return(() => call($18, "load8_u", [53_132]), 0);

// memory_copy.wast:3871
assert_return(() => call($18, "load8_u", [53_331]), 0);

// memory_copy.wast:3872
assert_return(() => call($18, "load8_u", [53_530]), 0);

// memory_copy.wast:3873
assert_return(() => call($18, "load8_u", [53_729]), 0);

// memory_copy.wast:3874
assert_return(() => call($18, "load8_u", [53_928]), 0);

// memory_copy.wast:3875
assert_return(() => call($18, "load8_u", [54_127]), 0);

// memory_copy.wast:3876
assert_return(() => call($18, "load8_u", [54_326]), 0);

// memory_copy.wast:3877
assert_return(() => call($18, "load8_u", [54_525]), 0);

// memory_copy.wast:3878
assert_return(() => call($18, "load8_u", [54_724]), 0);

// memory_copy.wast:3879
assert_return(() => call($18, "load8_u", [54_923]), 0);

// memory_copy.wast:3880
assert_return(() => call($18, "load8_u", [55_122]), 0);

// memory_copy.wast:3881
assert_return(() => call($18, "load8_u", [55_321]), 0);

// memory_copy.wast:3882
assert_return(() => call($18, "load8_u", [55_520]), 0);

// memory_copy.wast:3883
assert_return(() => call($18, "load8_u", [55_719]), 0);

// memory_copy.wast:3884
assert_return(() => call($18, "load8_u", [55_918]), 0);

// memory_copy.wast:3885
assert_return(() => call($18, "load8_u", [56_117]), 0);

// memory_copy.wast:3886
assert_return(() => call($18, "load8_u", [56_316]), 0);

// memory_copy.wast:3887
assert_return(() => call($18, "load8_u", [56_515]), 0);

// memory_copy.wast:3888
assert_return(() => call($18, "load8_u", [56_714]), 0);

// memory_copy.wast:3889
assert_return(() => call($18, "load8_u", [56_913]), 0);

// memory_copy.wast:3890
assert_return(() => call($18, "load8_u", [57_112]), 0);

// memory_copy.wast:3891
assert_return(() => call($18, "load8_u", [57_311]), 0);

// memory_copy.wast:3892
assert_return(() => call($18, "load8_u", [57_510]), 0);

// memory_copy.wast:3893
assert_return(() => call($18, "load8_u", [57_709]), 0);

// memory_copy.wast:3894
assert_return(() => call($18, "load8_u", [57_908]), 0);

// memory_copy.wast:3895
assert_return(() => call($18, "load8_u", [58_107]), 0);

// memory_copy.wast:3896
assert_return(() => call($18, "load8_u", [58_306]), 0);

// memory_copy.wast:3897
assert_return(() => call($18, "load8_u", [58_505]), 0);

// memory_copy.wast:3898
assert_return(() => call($18, "load8_u", [58_704]), 0);

// memory_copy.wast:3899
assert_return(() => call($18, "load8_u", [58_903]), 0);

// memory_copy.wast:3900
assert_return(() => call($18, "load8_u", [59_102]), 0);

// memory_copy.wast:3901
assert_return(() => call($18, "load8_u", [59_301]), 0);

// memory_copy.wast:3902
assert_return(() => call($18, "load8_u", [59_500]), 0);

// memory_copy.wast:3903
assert_return(() => call($18, "load8_u", [59_699]), 0);

// memory_copy.wast:3904
assert_return(() => call($18, "load8_u", [59_898]), 0);

// memory_copy.wast:3905
assert_return(() => call($18, "load8_u", [60_097]), 0);

// memory_copy.wast:3906
assert_return(() => call($18, "load8_u", [60_296]), 0);

// memory_copy.wast:3907
assert_return(() => call($18, "load8_u", [60_495]), 0);

// memory_copy.wast:3908
assert_return(() => call($18, "load8_u", [60_694]), 0);

// memory_copy.wast:3909
assert_return(() => call($18, "load8_u", [60_893]), 0);

// memory_copy.wast:3910
assert_return(() => call($18, "load8_u", [61_092]), 0);

// memory_copy.wast:3911
assert_return(() => call($18, "load8_u", [61_291]), 0);

// memory_copy.wast:3912
assert_return(() => call($18, "load8_u", [61_490]), 0);

// memory_copy.wast:3913
assert_return(() => call($18, "load8_u", [61_689]), 0);

// memory_copy.wast:3914
assert_return(() => call($18, "load8_u", [61_888]), 0);

// memory_copy.wast:3915
assert_return(() => call($18, "load8_u", [62_087]), 0);

// memory_copy.wast:3916
assert_return(() => call($18, "load8_u", [62_286]), 0);

// memory_copy.wast:3917
assert_return(() => call($18, "load8_u", [62_485]), 0);

// memory_copy.wast:3918
assert_return(() => call($18, "load8_u", [62_684]), 0);

// memory_copy.wast:3919
assert_return(() => call($18, "load8_u", [62_883]), 0);

// memory_copy.wast:3920
assert_return(() => call($18, "load8_u", [63_082]), 0);

// memory_copy.wast:3921
assert_return(() => call($18, "load8_u", [63_281]), 0);

// memory_copy.wast:3922
assert_return(() => call($18, "load8_u", [63_480]), 0);

// memory_copy.wast:3923
assert_return(() => call($18, "load8_u", [63_679]), 0);

// memory_copy.wast:3924
assert_return(() => call($18, "load8_u", [63_878]), 0);

// memory_copy.wast:3925
assert_return(() => call($18, "load8_u", [64_077]), 0);

// memory_copy.wast:3926
assert_return(() => call($18, "load8_u", [64_276]), 0);

// memory_copy.wast:3927
assert_return(() => call($18, "load8_u", [64_475]), 0);

// memory_copy.wast:3928
assert_return(() => call($18, "load8_u", [64_674]), 0);

// memory_copy.wast:3929
assert_return(() => call($18, "load8_u", [64_873]), 0);

// memory_copy.wast:3930
assert_return(() => call($18, "load8_u", [65_072]), 0);

// memory_copy.wast:3931
assert_return(() => call($18, "load8_u", [65_271]), 0);

// memory_copy.wast:3932
assert_return(() => call($18, "load8_u", [65_470]), 0);

// memory_copy.wast:3933
assert_return(() => call($18, "load8_u", [65_516]), 0);

// memory_copy.wast:3934
assert_return(() => call($18, "load8_u", [65_517]), 1);

// memory_copy.wast:3935
assert_return(() => call($18, "load8_u", [65_518]), 2);

// memory_copy.wast:3936
assert_return(() => call($18, "load8_u", [65_519]), 3);

// memory_copy.wast:3937
assert_return(() => call($18, "load8_u", [65_520]), 4);

// memory_copy.wast:3938
assert_return(() => call($18, "load8_u", [65_521]), 5);

// memory_copy.wast:3939
assert_return(() => call($18, "load8_u", [65_522]), 6);

// memory_copy.wast:3940
assert_return(() => call($18, "load8_u", [65_523]), 7);

// memory_copy.wast:3941
assert_return(() => call($18, "load8_u", [65_524]), 8);

// memory_copy.wast:3942
assert_return(() => call($18, "load8_u", [65_525]), 9);

// memory_copy.wast:3943
assert_return(() => call($18, "load8_u", [65_526]), 10);

// memory_copy.wast:3944
assert_return(() => call($18, "load8_u", [65_527]), 11);

// memory_copy.wast:3945
assert_return(() => call($18, "load8_u", [65_528]), 12);

// memory_copy.wast:3946
assert_return(() => call($18, "load8_u", [65_529]), 13);

// memory_copy.wast:3947
assert_return(() => call($18, "load8_u", [65_530]), 14);

// memory_copy.wast:3948
assert_return(() => call($18, "load8_u", [65_531]), 15);

// memory_copy.wast:3949
assert_return(() => call($18, "load8_u", [65_532]), 16);

// memory_copy.wast:3950
assert_return(() => call($18, "load8_u", [65_533]), 17);

// memory_copy.wast:3951
assert_return(() => call($18, "load8_u", [65_534]), 18);

// memory_copy.wast:3952
assert_return(() => call($18, "load8_u", [65_535]), 19);

// memory_copy.wast:3954
let $19 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x97\x80\x80\x80\x00\x03\x03\x6d\x65\x6d\x02\x00\x03\x72\x75\x6e\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x9c\x80\x80\x80\x00\x01\x00\x41\x80\xe0\x03\x0b\x14\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

// memory_copy.wast:3962
assert_trap(() => call($19, "run", [65_516, 61_440, -256]));

// memory_copy.wast:3965
assert_return(() => call($19, "load8_u", [198]), 0);

// memory_copy.wast:3966
assert_return(() => call($19, "load8_u", [397]), 0);

// memory_copy.wast:3967
assert_return(() => call($19, "load8_u", [596]), 0);

// memory_copy.wast:3968
assert_return(() => call($19, "load8_u", [795]), 0);

// memory_copy.wast:3969
assert_return(() => call($19, "load8_u", [994]), 0);

// memory_copy.wast:3970
assert_return(() => call($19, "load8_u", [1_193]), 0);

// memory_copy.wast:3971
assert_return(() => call($19, "load8_u", [1_392]), 0);

// memory_copy.wast:3972
assert_return(() => call($19, "load8_u", [1_591]), 0);

// memory_copy.wast:3973
assert_return(() => call($19, "load8_u", [1_790]), 0);

// memory_copy.wast:3974
assert_return(() => call($19, "load8_u", [1_989]), 0);

// memory_copy.wast:3975
assert_return(() => call($19, "load8_u", [2_188]), 0);

// memory_copy.wast:3976
assert_return(() => call($19, "load8_u", [2_387]), 0);

// memory_copy.wast:3977
assert_return(() => call($19, "load8_u", [2_586]), 0);

// memory_copy.wast:3978
assert_return(() => call($19, "load8_u", [2_785]), 0);

// memory_copy.wast:3979
assert_return(() => call($19, "load8_u", [2_984]), 0);

// memory_copy.wast:3980
assert_return(() => call($19, "load8_u", [3_183]), 0);

// memory_copy.wast:3981
assert_return(() => call($19, "load8_u", [3_382]), 0);

// memory_copy.wast:3982
assert_return(() => call($19, "load8_u", [3_581]), 0);

// memory_copy.wast:3983
assert_return(() => call($19, "load8_u", [3_780]), 0);

// memory_copy.wast:3984
assert_return(() => call($19, "load8_u", [3_979]), 0);

// memory_copy.wast:3985
assert_return(() => call($19, "load8_u", [4_178]), 0);

// memory_copy.wast:3986
assert_return(() => call($19, "load8_u", [4_377]), 0);

// memory_copy.wast:3987
assert_return(() => call($19, "load8_u", [4_576]), 0);

// memory_copy.wast:3988
assert_return(() => call($19, "load8_u", [4_775]), 0);

// memory_copy.wast:3989
assert_return(() => call($19, "load8_u", [4_974]), 0);

// memory_copy.wast:3990
assert_return(() => call($19, "load8_u", [5_173]), 0);

// memory_copy.wast:3991
assert_return(() => call($19, "load8_u", [5_372]), 0);

// memory_copy.wast:3992
assert_return(() => call($19, "load8_u", [5_571]), 0);

// memory_copy.wast:3993
assert_return(() => call($19, "load8_u", [5_770]), 0);

// memory_copy.wast:3994
assert_return(() => call($19, "load8_u", [5_969]), 0);

// memory_copy.wast:3995
assert_return(() => call($19, "load8_u", [6_168]), 0);

// memory_copy.wast:3996
assert_return(() => call($19, "load8_u", [6_367]), 0);

// memory_copy.wast:3997
assert_return(() => call($19, "load8_u", [6_566]), 0);

// memory_copy.wast:3998
assert_return(() => call($19, "load8_u", [6_765]), 0);

// memory_copy.wast:3999
assert_return(() => call($19, "load8_u", [6_964]), 0);

// memory_copy.wast:4000
assert_return(() => call($19, "load8_u", [7_163]), 0);

// memory_copy.wast:4001
assert_return(() => call($19, "load8_u", [7_362]), 0);

// memory_copy.wast:4002
assert_return(() => call($19, "load8_u", [7_561]), 0);

// memory_copy.wast:4003
assert_return(() => call($19, "load8_u", [7_760]), 0);

// memory_copy.wast:4004
assert_return(() => call($19, "load8_u", [7_959]), 0);

// memory_copy.wast:4005
assert_return(() => call($19, "load8_u", [8_158]), 0);

// memory_copy.wast:4006
assert_return(() => call($19, "load8_u", [8_357]), 0);

// memory_copy.wast:4007
assert_return(() => call($19, "load8_u", [8_556]), 0);

// memory_copy.wast:4008
assert_return(() => call($19, "load8_u", [8_755]), 0);

// memory_copy.wast:4009
assert_return(() => call($19, "load8_u", [8_954]), 0);

// memory_copy.wast:4010
assert_return(() => call($19, "load8_u", [9_153]), 0);

// memory_copy.wast:4011
assert_return(() => call($19, "load8_u", [9_352]), 0);

// memory_copy.wast:4012
assert_return(() => call($19, "load8_u", [9_551]), 0);

// memory_copy.wast:4013
assert_return(() => call($19, "load8_u", [9_750]), 0);

// memory_copy.wast:4014
assert_return(() => call($19, "load8_u", [9_949]), 0);

// memory_copy.wast:4015
assert_return(() => call($19, "load8_u", [10_148]), 0);

// memory_copy.wast:4016
assert_return(() => call($19, "load8_u", [10_347]), 0);

// memory_copy.wast:4017
assert_return(() => call($19, "load8_u", [10_546]), 0);

// memory_copy.wast:4018
assert_return(() => call($19, "load8_u", [10_745]), 0);

// memory_copy.wast:4019
assert_return(() => call($19, "load8_u", [10_944]), 0);

// memory_copy.wast:4020
assert_return(() => call($19, "load8_u", [11_143]), 0);

// memory_copy.wast:4021
assert_return(() => call($19, "load8_u", [11_342]), 0);

// memory_copy.wast:4022
assert_return(() => call($19, "load8_u", [11_541]), 0);

// memory_copy.wast:4023
assert_return(() => call($19, "load8_u", [11_740]), 0);

// memory_copy.wast:4024
assert_return(() => call($19, "load8_u", [11_939]), 0);

// memory_copy.wast:4025
assert_return(() => call($19, "load8_u", [12_138]), 0);

// memory_copy.wast:4026
assert_return(() => call($19, "load8_u", [12_337]), 0);

// memory_copy.wast:4027
assert_return(() => call($19, "load8_u", [12_536]), 0);

// memory_copy.wast:4028
assert_return(() => call($19, "load8_u", [12_735]), 0);

// memory_copy.wast:4029
assert_return(() => call($19, "load8_u", [12_934]), 0);

// memory_copy.wast:4030
assert_return(() => call($19, "load8_u", [13_133]), 0);

// memory_copy.wast:4031
assert_return(() => call($19, "load8_u", [13_332]), 0);

// memory_copy.wast:4032
assert_return(() => call($19, "load8_u", [13_531]), 0);

// memory_copy.wast:4033
assert_return(() => call($19, "load8_u", [13_730]), 0);

// memory_copy.wast:4034
assert_return(() => call($19, "load8_u", [13_929]), 0);

// memory_copy.wast:4035
assert_return(() => call($19, "load8_u", [14_128]), 0);

// memory_copy.wast:4036
assert_return(() => call($19, "load8_u", [14_327]), 0);

// memory_copy.wast:4037
assert_return(() => call($19, "load8_u", [14_526]), 0);

// memory_copy.wast:4038
assert_return(() => call($19, "load8_u", [14_725]), 0);

// memory_copy.wast:4039
assert_return(() => call($19, "load8_u", [14_924]), 0);

// memory_copy.wast:4040
assert_return(() => call($19, "load8_u", [15_123]), 0);

// memory_copy.wast:4041
assert_return(() => call($19, "load8_u", [15_322]), 0);

// memory_copy.wast:4042
assert_return(() => call($19, "load8_u", [15_521]), 0);

// memory_copy.wast:4043
assert_return(() => call($19, "load8_u", [15_720]), 0);

// memory_copy.wast:4044
assert_return(() => call($19, "load8_u", [15_919]), 0);

// memory_copy.wast:4045
assert_return(() => call($19, "load8_u", [16_118]), 0);

// memory_copy.wast:4046
assert_return(() => call($19, "load8_u", [16_317]), 0);

// memory_copy.wast:4047
assert_return(() => call($19, "load8_u", [16_516]), 0);

// memory_copy.wast:4048
assert_return(() => call($19, "load8_u", [16_715]), 0);

// memory_copy.wast:4049
assert_return(() => call($19, "load8_u", [16_914]), 0);

// memory_copy.wast:4050
assert_return(() => call($19, "load8_u", [17_113]), 0);

// memory_copy.wast:4051
assert_return(() => call($19, "load8_u", [17_312]), 0);

// memory_copy.wast:4052
assert_return(() => call($19, "load8_u", [17_511]), 0);

// memory_copy.wast:4053
assert_return(() => call($19, "load8_u", [17_710]), 0);

// memory_copy.wast:4054
assert_return(() => call($19, "load8_u", [17_909]), 0);

// memory_copy.wast:4055
assert_return(() => call($19, "load8_u", [18_108]), 0);

// memory_copy.wast:4056
assert_return(() => call($19, "load8_u", [18_307]), 0);

// memory_copy.wast:4057
assert_return(() => call($19, "load8_u", [18_506]), 0);

// memory_copy.wast:4058
assert_return(() => call($19, "load8_u", [18_705]), 0);

// memory_copy.wast:4059
assert_return(() => call($19, "load8_u", [18_904]), 0);

// memory_copy.wast:4060
assert_return(() => call($19, "load8_u", [19_103]), 0);

// memory_copy.wast:4061
assert_return(() => call($19, "load8_u", [19_302]), 0);

// memory_copy.wast:4062
assert_return(() => call($19, "load8_u", [19_501]), 0);

// memory_copy.wast:4063
assert_return(() => call($19, "load8_u", [19_700]), 0);

// memory_copy.wast:4064
assert_return(() => call($19, "load8_u", [19_899]), 0);

// memory_copy.wast:4065
assert_return(() => call($19, "load8_u", [20_098]), 0);

// memory_copy.wast:4066
assert_return(() => call($19, "load8_u", [20_297]), 0);

// memory_copy.wast:4067
assert_return(() => call($19, "load8_u", [20_496]), 0);

// memory_copy.wast:4068
assert_return(() => call($19, "load8_u", [20_695]), 0);

// memory_copy.wast:4069
assert_return(() => call($19, "load8_u", [20_894]), 0);

// memory_copy.wast:4070
assert_return(() => call($19, "load8_u", [21_093]), 0);

// memory_copy.wast:4071
assert_return(() => call($19, "load8_u", [21_292]), 0);

// memory_copy.wast:4072
assert_return(() => call($19, "load8_u", [21_491]), 0);

// memory_copy.wast:4073
assert_return(() => call($19, "load8_u", [21_690]), 0);

// memory_copy.wast:4074
assert_return(() => call($19, "load8_u", [21_889]), 0);

// memory_copy.wast:4075
assert_return(() => call($19, "load8_u", [22_088]), 0);

// memory_copy.wast:4076
assert_return(() => call($19, "load8_u", [22_287]), 0);

// memory_copy.wast:4077
assert_return(() => call($19, "load8_u", [22_486]), 0);

// memory_copy.wast:4078
assert_return(() => call($19, "load8_u", [22_685]), 0);

// memory_copy.wast:4079
assert_return(() => call($19, "load8_u", [22_884]), 0);

// memory_copy.wast:4080
assert_return(() => call($19, "load8_u", [23_083]), 0);

// memory_copy.wast:4081
assert_return(() => call($19, "load8_u", [23_282]), 0);

// memory_copy.wast:4082
assert_return(() => call($19, "load8_u", [23_481]), 0);

// memory_copy.wast:4083
assert_return(() => call($19, "load8_u", [23_680]), 0);

// memory_copy.wast:4084
assert_return(() => call($19, "load8_u", [23_879]), 0);

// memory_copy.wast:4085
assert_return(() => call($19, "load8_u", [24_078]), 0);

// memory_copy.wast:4086
assert_return(() => call($19, "load8_u", [24_277]), 0);

// memory_copy.wast:4087
assert_return(() => call($19, "load8_u", [24_476]), 0);

// memory_copy.wast:4088
assert_return(() => call($19, "load8_u", [24_675]), 0);

// memory_copy.wast:4089
assert_return(() => call($19, "load8_u", [24_874]), 0);

// memory_copy.wast:4090
assert_return(() => call($19, "load8_u", [25_073]), 0);

// memory_copy.wast:4091
assert_return(() => call($19, "load8_u", [25_272]), 0);

// memory_copy.wast:4092
assert_return(() => call($19, "load8_u", [25_471]), 0);

// memory_copy.wast:4093
assert_return(() => call($19, "load8_u", [25_670]), 0);

// memory_copy.wast:4094
assert_return(() => call($19, "load8_u", [25_869]), 0);

// memory_copy.wast:4095
assert_return(() => call($19, "load8_u", [26_068]), 0);

// memory_copy.wast:4096
assert_return(() => call($19, "load8_u", [26_267]), 0);

// memory_copy.wast:4097
assert_return(() => call($19, "load8_u", [26_466]), 0);

// memory_copy.wast:4098
assert_return(() => call($19, "load8_u", [26_665]), 0);

// memory_copy.wast:4099
assert_return(() => call($19, "load8_u", [26_864]), 0);

// memory_copy.wast:4100
assert_return(() => call($19, "load8_u", [27_063]), 0);

// memory_copy.wast:4101
assert_return(() => call($19, "load8_u", [27_262]), 0);

// memory_copy.wast:4102
assert_return(() => call($19, "load8_u", [27_461]), 0);

// memory_copy.wast:4103
assert_return(() => call($19, "load8_u", [27_660]), 0);

// memory_copy.wast:4104
assert_return(() => call($19, "load8_u", [27_859]), 0);

// memory_copy.wast:4105
assert_return(() => call($19, "load8_u", [28_058]), 0);

// memory_copy.wast:4106
assert_return(() => call($19, "load8_u", [28_257]), 0);

// memory_copy.wast:4107
assert_return(() => call($19, "load8_u", [28_456]), 0);

// memory_copy.wast:4108
assert_return(() => call($19, "load8_u", [28_655]), 0);

// memory_copy.wast:4109
assert_return(() => call($19, "load8_u", [28_854]), 0);

// memory_copy.wast:4110
assert_return(() => call($19, "load8_u", [29_053]), 0);

// memory_copy.wast:4111
assert_return(() => call($19, "load8_u", [29_252]), 0);

// memory_copy.wast:4112
assert_return(() => call($19, "load8_u", [29_451]), 0);

// memory_copy.wast:4113
assert_return(() => call($19, "load8_u", [29_650]), 0);

// memory_copy.wast:4114
assert_return(() => call($19, "load8_u", [29_849]), 0);

// memory_copy.wast:4115
assert_return(() => call($19, "load8_u", [30_048]), 0);

// memory_copy.wast:4116
assert_return(() => call($19, "load8_u", [30_247]), 0);

// memory_copy.wast:4117
assert_return(() => call($19, "load8_u", [30_446]), 0);

// memory_copy.wast:4118
assert_return(() => call($19, "load8_u", [30_645]), 0);

// memory_copy.wast:4119
assert_return(() => call($19, "load8_u", [30_844]), 0);

// memory_copy.wast:4120
assert_return(() => call($19, "load8_u", [31_043]), 0);

// memory_copy.wast:4121
assert_return(() => call($19, "load8_u", [31_242]), 0);

// memory_copy.wast:4122
assert_return(() => call($19, "load8_u", [31_441]), 0);

// memory_copy.wast:4123
assert_return(() => call($19, "load8_u", [31_640]), 0);

// memory_copy.wast:4124
assert_return(() => call($19, "load8_u", [31_839]), 0);

// memory_copy.wast:4125
assert_return(() => call($19, "load8_u", [32_038]), 0);

// memory_copy.wast:4126
assert_return(() => call($19, "load8_u", [32_237]), 0);

// memory_copy.wast:4127
assert_return(() => call($19, "load8_u", [32_436]), 0);

// memory_copy.wast:4128
assert_return(() => call($19, "load8_u", [32_635]), 0);

// memory_copy.wast:4129
assert_return(() => call($19, "load8_u", [32_834]), 0);

// memory_copy.wast:4130
assert_return(() => call($19, "load8_u", [33_033]), 0);

// memory_copy.wast:4131
assert_return(() => call($19, "load8_u", [33_232]), 0);

// memory_copy.wast:4132
assert_return(() => call($19, "load8_u", [33_431]), 0);

// memory_copy.wast:4133
assert_return(() => call($19, "load8_u", [33_630]), 0);

// memory_copy.wast:4134
assert_return(() => call($19, "load8_u", [33_829]), 0);

// memory_copy.wast:4135
assert_return(() => call($19, "load8_u", [34_028]), 0);

// memory_copy.wast:4136
assert_return(() => call($19, "load8_u", [34_227]), 0);

// memory_copy.wast:4137
assert_return(() => call($19, "load8_u", [34_426]), 0);

// memory_copy.wast:4138
assert_return(() => call($19, "load8_u", [34_625]), 0);

// memory_copy.wast:4139
assert_return(() => call($19, "load8_u", [34_824]), 0);

// memory_copy.wast:4140
assert_return(() => call($19, "load8_u", [35_023]), 0);

// memory_copy.wast:4141
assert_return(() => call($19, "load8_u", [35_222]), 0);

// memory_copy.wast:4142
assert_return(() => call($19, "load8_u", [35_421]), 0);

// memory_copy.wast:4143
assert_return(() => call($19, "load8_u", [35_620]), 0);

// memory_copy.wast:4144
assert_return(() => call($19, "load8_u", [35_819]), 0);

// memory_copy.wast:4145
assert_return(() => call($19, "load8_u", [36_018]), 0);

// memory_copy.wast:4146
assert_return(() => call($19, "load8_u", [36_217]), 0);

// memory_copy.wast:4147
assert_return(() => call($19, "load8_u", [36_416]), 0);

// memory_copy.wast:4148
assert_return(() => call($19, "load8_u", [36_615]), 0);

// memory_copy.wast:4149
assert_return(() => call($19, "load8_u", [36_814]), 0);

// memory_copy.wast:4150
assert_return(() => call($19, "load8_u", [37_013]), 0);

// memory_copy.wast:4151
assert_return(() => call($19, "load8_u", [37_212]), 0);

// memory_copy.wast:4152
assert_return(() => call($19, "load8_u", [37_411]), 0);

// memory_copy.wast:4153
assert_return(() => call($19, "load8_u", [37_610]), 0);

// memory_copy.wast:4154
assert_return(() => call($19, "load8_u", [37_809]), 0);

// memory_copy.wast:4155
assert_return(() => call($19, "load8_u", [38_008]), 0);

// memory_copy.wast:4156
assert_return(() => call($19, "load8_u", [38_207]), 0);

// memory_copy.wast:4157
assert_return(() => call($19, "load8_u", [38_406]), 0);

// memory_copy.wast:4158
assert_return(() => call($19, "load8_u", [38_605]), 0);

// memory_copy.wast:4159
assert_return(() => call($19, "load8_u", [38_804]), 0);

// memory_copy.wast:4160
assert_return(() => call($19, "load8_u", [39_003]), 0);

// memory_copy.wast:4161
assert_return(() => call($19, "load8_u", [39_202]), 0);

// memory_copy.wast:4162
assert_return(() => call($19, "load8_u", [39_401]), 0);

// memory_copy.wast:4163
assert_return(() => call($19, "load8_u", [39_600]), 0);

// memory_copy.wast:4164
assert_return(() => call($19, "load8_u", [39_799]), 0);

// memory_copy.wast:4165
assert_return(() => call($19, "load8_u", [39_998]), 0);

// memory_copy.wast:4166
assert_return(() => call($19, "load8_u", [40_197]), 0);

// memory_copy.wast:4167
assert_return(() => call($19, "load8_u", [40_396]), 0);

// memory_copy.wast:4168
assert_return(() => call($19, "load8_u", [40_595]), 0);

// memory_copy.wast:4169
assert_return(() => call($19, "load8_u", [40_794]), 0);

// memory_copy.wast:4170
assert_return(() => call($19, "load8_u", [40_993]), 0);

// memory_copy.wast:4171
assert_return(() => call($19, "load8_u", [41_192]), 0);

// memory_copy.wast:4172
assert_return(() => call($19, "load8_u", [41_391]), 0);

// memory_copy.wast:4173
assert_return(() => call($19, "load8_u", [41_590]), 0);

// memory_copy.wast:4174
assert_return(() => call($19, "load8_u", [41_789]), 0);

// memory_copy.wast:4175
assert_return(() => call($19, "load8_u", [41_988]), 0);

// memory_copy.wast:4176
assert_return(() => call($19, "load8_u", [42_187]), 0);

// memory_copy.wast:4177
assert_return(() => call($19, "load8_u", [42_386]), 0);

// memory_copy.wast:4178
assert_return(() => call($19, "load8_u", [42_585]), 0);

// memory_copy.wast:4179
assert_return(() => call($19, "load8_u", [42_784]), 0);

// memory_copy.wast:4180
assert_return(() => call($19, "load8_u", [42_983]), 0);

// memory_copy.wast:4181
assert_return(() => call($19, "load8_u", [43_182]), 0);

// memory_copy.wast:4182
assert_return(() => call($19, "load8_u", [43_381]), 0);

// memory_copy.wast:4183
assert_return(() => call($19, "load8_u", [43_580]), 0);

// memory_copy.wast:4184
assert_return(() => call($19, "load8_u", [43_779]), 0);

// memory_copy.wast:4185
assert_return(() => call($19, "load8_u", [43_978]), 0);

// memory_copy.wast:4186
assert_return(() => call($19, "load8_u", [44_177]), 0);

// memory_copy.wast:4187
assert_return(() => call($19, "load8_u", [44_376]), 0);

// memory_copy.wast:4188
assert_return(() => call($19, "load8_u", [44_575]), 0);

// memory_copy.wast:4189
assert_return(() => call($19, "load8_u", [44_774]), 0);

// memory_copy.wast:4190
assert_return(() => call($19, "load8_u", [44_973]), 0);

// memory_copy.wast:4191
assert_return(() => call($19, "load8_u", [45_172]), 0);

// memory_copy.wast:4192
assert_return(() => call($19, "load8_u", [45_371]), 0);

// memory_copy.wast:4193
assert_return(() => call($19, "load8_u", [45_570]), 0);

// memory_copy.wast:4194
assert_return(() => call($19, "load8_u", [45_769]), 0);

// memory_copy.wast:4195
assert_return(() => call($19, "load8_u", [45_968]), 0);

// memory_copy.wast:4196
assert_return(() => call($19, "load8_u", [46_167]), 0);

// memory_copy.wast:4197
assert_return(() => call($19, "load8_u", [46_366]), 0);

// memory_copy.wast:4198
assert_return(() => call($19, "load8_u", [46_565]), 0);

// memory_copy.wast:4199
assert_return(() => call($19, "load8_u", [46_764]), 0);

// memory_copy.wast:4200
assert_return(() => call($19, "load8_u", [46_963]), 0);

// memory_copy.wast:4201
assert_return(() => call($19, "load8_u", [47_162]), 0);

// memory_copy.wast:4202
assert_return(() => call($19, "load8_u", [47_361]), 0);

// memory_copy.wast:4203
assert_return(() => call($19, "load8_u", [47_560]), 0);

// memory_copy.wast:4204
assert_return(() => call($19, "load8_u", [47_759]), 0);

// memory_copy.wast:4205
assert_return(() => call($19, "load8_u", [47_958]), 0);

// memory_copy.wast:4206
assert_return(() => call($19, "load8_u", [48_157]), 0);

// memory_copy.wast:4207
assert_return(() => call($19, "load8_u", [48_356]), 0);

// memory_copy.wast:4208
assert_return(() => call($19, "load8_u", [48_555]), 0);

// memory_copy.wast:4209
assert_return(() => call($19, "load8_u", [48_754]), 0);

// memory_copy.wast:4210
assert_return(() => call($19, "load8_u", [48_953]), 0);

// memory_copy.wast:4211
assert_return(() => call($19, "load8_u", [49_152]), 0);

// memory_copy.wast:4212
assert_return(() => call($19, "load8_u", [49_351]), 0);

// memory_copy.wast:4213
assert_return(() => call($19, "load8_u", [49_550]), 0);

// memory_copy.wast:4214
assert_return(() => call($19, "load8_u", [49_749]), 0);

// memory_copy.wast:4215
assert_return(() => call($19, "load8_u", [49_948]), 0);

// memory_copy.wast:4216
assert_return(() => call($19, "load8_u", [50_147]), 0);

// memory_copy.wast:4217
assert_return(() => call($19, "load8_u", [50_346]), 0);

// memory_copy.wast:4218
assert_return(() => call($19, "load8_u", [50_545]), 0);

// memory_copy.wast:4219
assert_return(() => call($19, "load8_u", [50_744]), 0);

// memory_copy.wast:4220
assert_return(() => call($19, "load8_u", [50_943]), 0);

// memory_copy.wast:4221
assert_return(() => call($19, "load8_u", [51_142]), 0);

// memory_copy.wast:4222
assert_return(() => call($19, "load8_u", [51_341]), 0);

// memory_copy.wast:4223
assert_return(() => call($19, "load8_u", [51_540]), 0);

// memory_copy.wast:4224
assert_return(() => call($19, "load8_u", [51_739]), 0);

// memory_copy.wast:4225
assert_return(() => call($19, "load8_u", [51_938]), 0);

// memory_copy.wast:4226
assert_return(() => call($19, "load8_u", [52_137]), 0);

// memory_copy.wast:4227
assert_return(() => call($19, "load8_u", [52_336]), 0);

// memory_copy.wast:4228
assert_return(() => call($19, "load8_u", [52_535]), 0);

// memory_copy.wast:4229
assert_return(() => call($19, "load8_u", [52_734]), 0);

// memory_copy.wast:4230
assert_return(() => call($19, "load8_u", [52_933]), 0);

// memory_copy.wast:4231
assert_return(() => call($19, "load8_u", [53_132]), 0);

// memory_copy.wast:4232
assert_return(() => call($19, "load8_u", [53_331]), 0);

// memory_copy.wast:4233
assert_return(() => call($19, "load8_u", [53_530]), 0);

// memory_copy.wast:4234
assert_return(() => call($19, "load8_u", [53_729]), 0);

// memory_copy.wast:4235
assert_return(() => call($19, "load8_u", [53_928]), 0);

// memory_copy.wast:4236
assert_return(() => call($19, "load8_u", [54_127]), 0);

// memory_copy.wast:4237
assert_return(() => call($19, "load8_u", [54_326]), 0);

// memory_copy.wast:4238
assert_return(() => call($19, "load8_u", [54_525]), 0);

// memory_copy.wast:4239
assert_return(() => call($19, "load8_u", [54_724]), 0);

// memory_copy.wast:4240
assert_return(() => call($19, "load8_u", [54_923]), 0);

// memory_copy.wast:4241
assert_return(() => call($19, "load8_u", [55_122]), 0);

// memory_copy.wast:4242
assert_return(() => call($19, "load8_u", [55_321]), 0);

// memory_copy.wast:4243
assert_return(() => call($19, "load8_u", [55_520]), 0);

// memory_copy.wast:4244
assert_return(() => call($19, "load8_u", [55_719]), 0);

// memory_copy.wast:4245
assert_return(() => call($19, "load8_u", [55_918]), 0);

// memory_copy.wast:4246
assert_return(() => call($19, "load8_u", [56_117]), 0);

// memory_copy.wast:4247
assert_return(() => call($19, "load8_u", [56_316]), 0);

// memory_copy.wast:4248
assert_return(() => call($19, "load8_u", [56_515]), 0);

// memory_copy.wast:4249
assert_return(() => call($19, "load8_u", [56_714]), 0);

// memory_copy.wast:4250
assert_return(() => call($19, "load8_u", [56_913]), 0);

// memory_copy.wast:4251
assert_return(() => call($19, "load8_u", [57_112]), 0);

// memory_copy.wast:4252
assert_return(() => call($19, "load8_u", [57_311]), 0);

// memory_copy.wast:4253
assert_return(() => call($19, "load8_u", [57_510]), 0);

// memory_copy.wast:4254
assert_return(() => call($19, "load8_u", [57_709]), 0);

// memory_copy.wast:4255
assert_return(() => call($19, "load8_u", [57_908]), 0);

// memory_copy.wast:4256
assert_return(() => call($19, "load8_u", [58_107]), 0);

// memory_copy.wast:4257
assert_return(() => call($19, "load8_u", [58_306]), 0);

// memory_copy.wast:4258
assert_return(() => call($19, "load8_u", [58_505]), 0);

// memory_copy.wast:4259
assert_return(() => call($19, "load8_u", [58_704]), 0);

// memory_copy.wast:4260
assert_return(() => call($19, "load8_u", [58_903]), 0);

// memory_copy.wast:4261
assert_return(() => call($19, "load8_u", [59_102]), 0);

// memory_copy.wast:4262
assert_return(() => call($19, "load8_u", [59_301]), 0);

// memory_copy.wast:4263
assert_return(() => call($19, "load8_u", [59_500]), 0);

// memory_copy.wast:4264
assert_return(() => call($19, "load8_u", [59_699]), 0);

// memory_copy.wast:4265
assert_return(() => call($19, "load8_u", [59_898]), 0);

// memory_copy.wast:4266
assert_return(() => call($19, "load8_u", [60_097]), 0);

// memory_copy.wast:4267
assert_return(() => call($19, "load8_u", [60_296]), 0);

// memory_copy.wast:4268
assert_return(() => call($19, "load8_u", [60_495]), 0);

// memory_copy.wast:4269
assert_return(() => call($19, "load8_u", [60_694]), 0);

// memory_copy.wast:4270
assert_return(() => call($19, "load8_u", [60_893]), 0);

// memory_copy.wast:4271
assert_return(() => call($19, "load8_u", [61_092]), 0);

// memory_copy.wast:4272
assert_return(() => call($19, "load8_u", [61_291]), 0);

// memory_copy.wast:4273
assert_return(() => call($19, "load8_u", [61_440]), 0);

// memory_copy.wast:4274
assert_return(() => call($19, "load8_u", [61_441]), 1);

// memory_copy.wast:4275
assert_return(() => call($19, "load8_u", [61_442]), 2);

// memory_copy.wast:4276
assert_return(() => call($19, "load8_u", [61_443]), 3);

// memory_copy.wast:4277
assert_return(() => call($19, "load8_u", [61_444]), 4);

// memory_copy.wast:4278
assert_return(() => call($19, "load8_u", [61_445]), 5);

// memory_copy.wast:4279
assert_return(() => call($19, "load8_u", [61_446]), 6);

// memory_copy.wast:4280
assert_return(() => call($19, "load8_u", [61_447]), 7);

// memory_copy.wast:4281
assert_return(() => call($19, "load8_u", [61_448]), 8);

// memory_copy.wast:4282
assert_return(() => call($19, "load8_u", [61_449]), 9);

// memory_copy.wast:4283
assert_return(() => call($19, "load8_u", [61_450]), 10);

// memory_copy.wast:4284
assert_return(() => call($19, "load8_u", [61_451]), 11);

// memory_copy.wast:4285
assert_return(() => call($19, "load8_u", [61_452]), 12);

// memory_copy.wast:4286
assert_return(() => call($19, "load8_u", [61_453]), 13);

// memory_copy.wast:4287
assert_return(() => call($19, "load8_u", [61_454]), 14);

// memory_copy.wast:4288
assert_return(() => call($19, "load8_u", [61_455]), 15);

// memory_copy.wast:4289
assert_return(() => call($19, "load8_u", [61_456]), 16);

// memory_copy.wast:4290
assert_return(() => call($19, "load8_u", [61_457]), 17);

// memory_copy.wast:4291
assert_return(() => call($19, "load8_u", [61_458]), 18);

// memory_copy.wast:4292
assert_return(() => call($19, "load8_u", [61_459]), 19);

// memory_copy.wast:4293
assert_return(() => call($19, "load8_u", [61_510]), 0);

// memory_copy.wast:4294
assert_return(() => call($19, "load8_u", [61_709]), 0);

// memory_copy.wast:4295
assert_return(() => call($19, "load8_u", [61_908]), 0);

// memory_copy.wast:4296
assert_return(() => call($19, "load8_u", [62_107]), 0);

// memory_copy.wast:4297
assert_return(() => call($19, "load8_u", [62_306]), 0);

// memory_copy.wast:4298
assert_return(() => call($19, "load8_u", [62_505]), 0);

// memory_copy.wast:4299
assert_return(() => call($19, "load8_u", [62_704]), 0);

// memory_copy.wast:4300
assert_return(() => call($19, "load8_u", [62_903]), 0);

// memory_copy.wast:4301
assert_return(() => call($19, "load8_u", [63_102]), 0);

// memory_copy.wast:4302
assert_return(() => call($19, "load8_u", [63_301]), 0);

// memory_copy.wast:4303
assert_return(() => call($19, "load8_u", [63_500]), 0);

// memory_copy.wast:4304
assert_return(() => call($19, "load8_u", [63_699]), 0);

// memory_copy.wast:4305
assert_return(() => call($19, "load8_u", [63_898]), 0);

// memory_copy.wast:4306
assert_return(() => call($19, "load8_u", [64_097]), 0);

// memory_copy.wast:4307
assert_return(() => call($19, "load8_u", [64_296]), 0);

// memory_copy.wast:4308
assert_return(() => call($19, "load8_u", [64_495]), 0);

// memory_copy.wast:4309
assert_return(() => call($19, "load8_u", [64_694]), 0);

// memory_copy.wast:4310
assert_return(() => call($19, "load8_u", [64_893]), 0);

// memory_copy.wast:4311
assert_return(() => call($19, "load8_u", [65_092]), 0);

// memory_copy.wast:4312
assert_return(() => call($19, "load8_u", [65_291]), 0);

// memory_copy.wast:4313
assert_return(() => call($19, "load8_u", [65_490]), 0);

// memory_copy.wast:4315
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x41\x0a\x41\x14\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4321
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x41\x0a\x41\x14\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4328
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x41\x0a\x41\x14\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4335
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x41\x0a\x41\x14\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4342
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x41\x0a\x43\x00\x00\xa0\x41\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4349
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x98\x80\x80\x80\x00\x01\x92\x80\x80\x80\x00\x00\x41\x0a\x43\x00\x00\xa0\x41\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4356
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x41\x0a\x43\x00\x00\xa0\x41\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4363
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x41\x0a\x43\x00\x00\xa0\x41\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4370
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x41\x0a\x42\x14\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4377
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x41\x0a\x42\x14\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4384
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x41\x0a\x42\x14\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4391
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x41\x0a\x42\x14\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4398
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x41\x0a\x44\x00\x00\x00\x00\x00\x00\x34\x40\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4405
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x41\x0a\x44\x00\x00\x00\x00\x00\x00\x34\x40\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4412
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x41\x0a\x44\x00\x00\x00\x00\x00\x00\x34\x40\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4419
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa0\x80\x80\x80\x00\x01\x9a\x80\x80\x80\x00\x00\x41\x0a\x44\x00\x00\x00\x00\x00\x00\x34\x40\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4426
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x41\x14\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4433
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x98\x80\x80\x80\x00\x01\x92\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x41\x14\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4440
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x41\x14\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4447
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x41\x14\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4454
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x98\x80\x80\x80\x00\x01\x92\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x43\x00\x00\xa0\x41\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4461
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9b\x80\x80\x80\x00\x01\x95\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x43\x00\x00\xa0\x41\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4468
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x98\x80\x80\x80\x00\x01\x92\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x43\x00\x00\xa0\x41\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4475
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9f\x80\x80\x80\x00\x01\x99\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x43\x00\x00\xa0\x41\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4482
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x42\x14\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4489
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x98\x80\x80\x80\x00\x01\x92\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x42\x14\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4496
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x42\x14\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4503
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x42\x14\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4510
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x44\x00\x00\x00\x00\x00\x00\x34\x40\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4517
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9f\x80\x80\x80\x00\x01\x99\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x44\x00\x00\x00\x00\x00\x00\x34\x40\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4524
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x44\x00\x00\x00\x00\x00\x00\x34\x40\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4531
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa3\x80\x80\x80\x00\x01\x9d\x80\x80\x80\x00\x00\x43\x00\x00\x20\x41\x44\x00\x00\x00\x00\x00\x00\x34\x40\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4538
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x42\x0a\x41\x14\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4545
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x42\x0a\x41\x14\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4552
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x42\x0a\x41\x14\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4559
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x42\x0a\x41\x14\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4566
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x42\x0a\x43\x00\x00\xa0\x41\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4573
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x98\x80\x80\x80\x00\x01\x92\x80\x80\x80\x00\x00\x42\x0a\x43\x00\x00\xa0\x41\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4580
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x42\x0a\x43\x00\x00\xa0\x41\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4587
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x42\x0a\x43\x00\x00\xa0\x41\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4594
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x42\x0a\x42\x14\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4601
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x95\x80\x80\x80\x00\x01\x8f\x80\x80\x80\x00\x00\x42\x0a\x42\x14\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4608
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x42\x0a\x42\x14\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4615
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x42\x0a\x42\x14\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4622
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x42\x0a\x44\x00\x00\x00\x00\x00\x00\x34\x40\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4629
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x42\x0a\x44\x00\x00\x00\x00\x00\x00\x34\x40\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4636
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x42\x0a\x44\x00\x00\x00\x00\x00\x00\x34\x40\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4643
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa0\x80\x80\x80\x00\x01\x9a\x80\x80\x80\x00\x00\x42\x0a\x44\x00\x00\x00\x00\x00\x00\x34\x40\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4650
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x41\x14\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4657
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x41\x14\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4664
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x41\x14\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4671
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa0\x80\x80\x80\x00\x01\x9a\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x41\x14\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4678
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x43\x00\x00\xa0\x41\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4685
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9f\x80\x80\x80\x00\x01\x99\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x43\x00\x00\xa0\x41\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4692
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x43\x00\x00\xa0\x41\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4699
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa3\x80\x80\x80\x00\x01\x9d\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x43\x00\x00\xa0\x41\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4706
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x42\x14\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4713
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x9c\x80\x80\x80\x00\x01\x96\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x42\x14\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4720
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\x99\x80\x80\x80\x00\x01\x93\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x42\x14\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4727
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa0\x80\x80\x80\x00\x01\x9a\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x42\x14\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4734
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa0\x80\x80\x80\x00\x01\x9a\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x44\x00\x00\x00\x00\x00\x00\x34\x40\x41\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4741
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa3\x80\x80\x80\x00\x01\x9d\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x44\x00\x00\x00\x00\x00\x00\x34\x40\x43\x00\x00\xf0\x41\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4748
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa0\x80\x80\x80\x00\x01\x9a\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x44\x00\x00\x00\x00\x00\x00\x34\x40\x42\x1e\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4755
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x8a\x80\x80\x80\x00\x01\x06\x74\x65\x73\x74\x66\x6e\x00\x00\x0a\xa7\x80\x80\x80\x00\x01\xa1\x80\x80\x80\x00\x00\x44\x00\x00\x00\x00\x00\x00\x24\x40\x44\x00\x00\x00\x00\x00\x00\x34\x40\x44\x00\x00\x00\x00\x00\x00\x3e\x40\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4763
let $20 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8b\x80\x80\x80\x00\x02\x60\x00\x00\x60\x03\x7f\x7f\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x95\x80\x80\x80\x00\x02\x04\x74\x65\x73\x74\x00\x00\x0a\x63\x68\x65\x63\x6b\x52\x61\x6e\x67\x65\x00\x01\x0a\xc8\x80\x80\x80\x00\x02\x96\x80\x80\x80\x00\x00\x41\x0a\x41\xd5\x00\x41\x0a\xfc\x0b\x00\x41\x09\x41\x0a\x41\x05\xfc\x0a\x00\x00\x0b\xa7\x80\x80\x80\x00\x00\x03\x40\x20\x00\x20\x01\x46\x04\x40\x41\x7f\x0f\x0b\x20\x00\x2d\x00\x00\x20\x02\x46\x04\x40\x20\x00\x41\x01\x6a\x21\x00\x0c\x01\x0b\x0b\x20\x00\x0f\x0b");

// memory_copy.wast:4780
run(() => call($20, "test", []));

// memory_copy.wast:4782
assert_return(() => call($20, "checkRange", [0, 9, 0]), -1);

// memory_copy.wast:4784
assert_return(() => call($20, "checkRange", [9, 20, 85]), -1);

// memory_copy.wast:4786
assert_return(() => call($20, "checkRange", [20, 65_536, 0]), -1);

// memory_copy.wast:4789
let $21 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8b\x80\x80\x80\x00\x02\x60\x00\x00\x60\x03\x7f\x7f\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x95\x80\x80\x80\x00\x02\x04\x74\x65\x73\x74\x00\x00\x0a\x63\x68\x65\x63\x6b\x52\x61\x6e\x67\x65\x00\x01\x0a\xc8\x80\x80\x80\x00\x02\x96\x80\x80\x80\x00\x00\x41\x0a\x41\xd5\x00\x41\x0a\xfc\x0b\x00\x41\x10\x41\x0f\x41\x05\xfc\x0a\x00\x00\x0b\xa7\x80\x80\x80\x00\x00\x03\x40\x20\x00\x20\x01\x46\x04\x40\x41\x7f\x0f\x0b\x20\x00\x2d\x00\x00\x20\x02\x46\x04\x40\x20\x00\x41\x01\x6a\x21\x00\x0c\x01\x0b\x0b\x20\x00\x0f\x0b");

// memory_copy.wast:4806
run(() => call($21, "test", []));

// memory_copy.wast:4808
assert_return(() => call($21, "checkRange", [0, 10, 0]), -1);

// memory_copy.wast:4810
assert_return(() => call($21, "checkRange", [10, 21, 85]), -1);

// memory_copy.wast:4812
assert_return(() => call($21, "checkRange", [21, 65_536, 0]), -1);

// memory_copy.wast:4815
let $22 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x97\x80\x80\x80\x00\x01\x91\x80\x80\x80\x00\x00\x41\x80\xfe\x03\x41\x80\x80\x02\x41\x81\x02\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4819
assert_trap(() => call($22, "test", []));

// memory_copy.wast:4821
let $23 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x96\x80\x80\x80\x00\x01\x90\x80\x80\x80\x00\x00\x41\x80\x7e\x41\x80\x80\x01\x41\x81\x02\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4825
assert_trap(() => call($23, "test", []));

// memory_copy.wast:4827
let $24 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x97\x80\x80\x80\x00\x01\x91\x80\x80\x80\x00\x00\x41\x80\x80\x02\x41\x80\xfe\x03\x41\x81\x02\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4831
assert_trap(() => call($24, "test", []));

// memory_copy.wast:4833
let $25 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x96\x80\x80\x80\x00\x01\x90\x80\x80\x80\x00\x00\x41\x80\x80\x01\x41\x80\x7e\x41\x81\x02\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4837
assert_trap(() => call($25, "test", []));

// memory_copy.wast:4839
let $26 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8b\x80\x80\x80\x00\x02\x60\x00\x00\x60\x03\x7f\x7f\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x95\x80\x80\x80\x00\x02\x04\x74\x65\x73\x74\x00\x00\x0a\x63\x68\x65\x63\x6b\x52\x61\x6e\x67\x65\x00\x01\x0a\xdc\x80\x80\x80\x00\x02\xaa\x80\x80\x80\x00\x00\x41\x00\x41\xd5\x00\x41\x80\x80\x02\xfc\x0b\x00\x41\x80\x80\x02\x41\xaa\x01\x41\x80\x80\x02\xfc\x0b\x00\x41\x80\xa0\x02\x41\x80\xe0\x01\x41\x00\xfc\x0a\x00\x00\x0b\xa7\x80\x80\x80\x00\x00\x03\x40\x20\x00\x20\x01\x46\x04\x40\x41\x7f\x0f\x0b\x20\x00\x2d\x00\x00\x20\x02\x46\x04\x40\x20\x00\x41\x01\x6a\x21\x00\x0c\x01\x0b\x0b\x20\x00\x0f\x0b");

// memory_copy.wast:4857
run(() => call($26, "test", []));

// memory_copy.wast:4859
assert_return(() => call($26, "checkRange", [0, 32_768, 85]), -1);

// memory_copy.wast:4861
assert_return(() => call($26, "checkRange", [32_768, 65_536, 170]), -1);

// memory_copy.wast:4863
let $27 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x96\x80\x80\x80\x00\x01\x90\x80\x80\x80\x00\x00\x41\x80\x80\x04\x41\x80\xe0\x01\x41\x00\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4867
run(() => call($27, "test", []));

// memory_copy.wast:4869
let $28 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x96\x80\x80\x80\x00\x01\x90\x80\x80\x80\x00\x00\x41\x80\x80\x08\x41\x80\xe0\x01\x41\x00\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4873
assert_trap(() => call($28, "test", []));

// memory_copy.wast:4875
let $29 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x96\x80\x80\x80\x00\x01\x90\x80\x80\x80\x00\x00\x41\x80\xa0\x02\x41\x80\x80\x04\x41\x00\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4879
run(() => call($29, "test", []));

// memory_copy.wast:4881
let $30 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x96\x80\x80\x80\x00\x01\x90\x80\x80\x80\x00\x00\x41\x80\xa0\x02\x41\x80\x80\x08\x41\x00\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4885
assert_trap(() => call($30, "test", []));

// memory_copy.wast:4887
let $31 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x96\x80\x80\x80\x00\x01\x90\x80\x80\x80\x00\x00\x41\x80\x80\x04\x41\x80\x80\x04\x41\x00\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4891
run(() => call($31, "test", []));

// memory_copy.wast:4893
let $32 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x88\x80\x80\x80\x00\x01\x04\x74\x65\x73\x74\x00\x00\x0a\x96\x80\x80\x80\x00\x01\x90\x80\x80\x80\x00\x00\x41\x80\x80\x08\x41\x80\x80\x08\x41\x00\xfc\x0a\x00\x00\x0b");

// memory_copy.wast:4897
assert_trap(() => call($32, "test", []));

// memory_copy.wast:4899
let $33 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8b\x80\x80\x80\x00\x02\x60\x00\x00\x60\x03\x7f\x7f\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x01\x01\x01\x07\x95\x80\x80\x80\x00\x02\x04\x74\x65\x73\x74\x00\x00\x0a\x63\x68\x65\x63\x6b\x52\x61\x6e\x67\x65\x00\x01\x0a\xbe\x95\x80\x80\x00\x02\x8c\x95\x80\x80\x00\x00\x41\xe7\x8a\x01\x41\x01\x41\xc0\x0a\xfc\x0b\x00\x41\xe9\xb0\x02\x41\x02\x41\x9f\x08\xfc\x0b\x00\x41\xd1\xb8\x03\x41\x03\x41\xdc\x07\xfc\x0b\x00\x41\xca\xa8\x02\x41\x04\x41\xc2\x02\xfc\x0b\x00\x41\xa9\x3e\x41\x05\x41\xca\x0f\xfc\x0b\x00\x41\xba\xb1\x01\x41\x06\x41\xdc\x17\xfc\x0b\x00\x41\xf2\x83\x01\x41\x07\x41\xc4\x12\xfc\x0b\x00\x41\xe3\xd3\x02\x41\x08\x41\xc3\x06\xfc\x0b\x00\x41\xfc\x00\x41\x09\x41\xf1\x0a\xfc\x0b\x00\x41\xd4\x10\x41\x0a\x41\xc6\x15\xfc\x0b\x00\x41\x9b\xc6\x00\x41\x0b\x41\x9a\x18\xfc\x0b\x00\x41\xe7\x9b\x03\x41\x0c\x41\xe5\x05\xfc\x0b\x00\x41\xf6\x1e\x41\x0d\x41\x87\x16\xfc\x0b\x00\x41\xb3\x84\x03\x41\x0e\x41\x80\x0a\xfc\x0b\x00\x41\xc9\x89\x03\x41\x0f\x41\xba\x0b\xfc\x0b\x00\x41\x8d\xa0\x01\x41\x10\x41\xd6\x18\xfc\x0b\x00\x41\xb1\xf4\x02\x41\x11\x41\xa0\x04\xfc\x0b\x00\x41\xa3\xe1\x00\x41\x12\x41\xed\x14\xfc\x0b\x00\x41\xa5\xc2\x01\x41\x13\x41\xdb\x14\xfc\x0b\x00\x41\x85\xe2\x02\x41\x14\x41\xa2\x0c\xfc\x0b\x00\x41\xd8\xd0\x02\x41\x15\x41\x9b\x0d\xfc\x0b\x00\x41\xde\x88\x02\x41\x16\x41\x86\x05\xfc\x0b\x00\x41\xab\xfb\x02\x41\x17\x41\xc2\x0e\xfc\x0b\x00\x41\xcd\xa1\x03\x41\x18\x41\xe1\x14\xfc\x0b\x00\x41\x9b\xed\x01\x41\x19\x41\xd5\x07\xfc\x0b\x00\x41\xd4\xc8\x00\x41\x1a\x41\x8f\x0e\xfc\x0b\x00\x41\x8e\x88\x03\x41\x1b\x41\xe7\x03\xfc\x0b\x00\x41\xa1\xea\x03\x41\x1c\x41\x92\x04\xfc\x0b\x00\x41\xdc\x9b\x02\x41\x1d\x41\xaf\x07\xfc\x0b\x00\x41\xf0\x34\x41\x1e\x41\xfd\x02\xfc\x0b\x00\x41\xbe\x90\x03\x41\x1f\x41\x91\x18\xfc\x0b\x00\x41\xc1\x84\x03\x41\x20\x41\x92\x05\xfc\x0b\x00\x41\xfc\xdb\x02\x41\x21\x41\xa6\x0d\xfc\x0b\x00\x41\xbe\x84\x02\x41\x22\x41\xc4\x08\xfc\x0b\x00\x41\xfe\x8c\x03\x41\x23\x41\x82\x0b\xfc\x0b\x00\x41\xea\xf3\x02\x41\x24\x41\x9c\x11\xfc\x0b\x00\x41\xeb\xa6\x03\x41\x25\x41\xda\x12\xfc\x0b\x00\x41\x8f\xaf\x03\x41\x26\x41\xfa\x01\xfc\x0b\x00\x41\xdc\xb0\x01\x41\x27\x41\xb1\x10\xfc\x0b\x00\x41\xec\x85\x01\x41\x28\x41\xc0\x19\xfc\x0b\x00\x41\xbb\xa8\x03\x41\x29\x41\xe3\x19\xfc\x0b\x00\x41\xb2\xb4\x02\x41\x2a\x41\xec\x15\xfc\x0b\x00\x41\xbc\x9a\x02\x41\x2b\x41\x96\x10\xfc\x0b\x00\x41\xec\x93\x02\x41\x2c\x41\xcb\x15\xfc\x0b\x00\x41\xdb\xff\x01\x41\x2d\x41\xb8\x02\xfc\x0b\x00\x41\x82\xf2\x03\x41\x2e\x41\xc0\x01\xfc\x0b\x00\x41\xfe\xf1\x01\x41\x2f\x41\xd4\x04\xfc\x0b\x00\x41\xfb\x81\x01\x41\x30\x41\xf5\x03\xfc\x0b\x00\x41\xaa\xbd\x03\x41\x31\x41\xae\x05\xfc\x0b\x00\x41\xfb\x8b\x02\x41\x32\x41\x81\x03\xfc\x0b\x00\x41\xd1\xdb\x03\x41\x33\x41\x87\x07\xfc\x0b\x00\x41\x85\xe0\x03\x41\x34\x41\xd6\x12\xfc\x0b\x00\x41\xfc\xee\x02\x41\x35\x41\xa1\x0b\xfc\x0b\x00\x41\xf5\xca\x01\x41\x36\x41\xda\x18\xfc\x0b\x00\x41\xbe\x2b\x41\x37\x41\xd7\x10\xfc\x0b\x00\x41\x89\x99\x02\x41\x38\x41\x87\x04\xfc\x0b\x00\x41\xdc\xde\x02\x41\x39\x41\xd0\x19\xfc\x0b\x00\x41\xa8\xed\x02\x41\x3a\x41\x8e\x0d\xfc\x0b\x00\x41\x8f\xec\x02\x41\x3b\x41\xe0\x18\xfc\x0b\x00\x41\xb1\xaf\x01\x41\x3c\x41\xa1\x0b\xfc\x0b\x00\x41\xf1\xc9\x03\x41\x3d\x41\x97\x05\xfc\x0b\x00\x41\x85\xfc\x01\x41\x3e\x41\x87\x0d\xfc\x0b\x00\x41\xf7\x17\x41\x3f\x41\xd1\x05\xfc\x0b\x00\x41\xe9\x89\x02\x41\xc0\x00\x41\xd4\x00\xfc\x0b\x00\x41\xba\x84\x02\x41\xc1\x00\x41\xed\x0f\xfc\x0b\x00\x41\xca\x9f\x02\x41\xc2\x00\x41\x1d\xfc\x0b\x00\x41\xcb\x95\x01\x41\xc3\x00\x41\xda\x17\xfc\x0b\x00\x41\xc8\xe2\x00\x41\xc4\x00\x41\x93\x08\xfc\x0b\x00\x41\xe4\x8e\x01\x41\xc5\x00\x41\xfc\x19\xfc\x0b\x00\x41\x9f\x24\x41\xc6\x00\x41\xc3\x08\xfc\x0b\x00\x41\x9e\xfe\x00\x41\xc7\x00\x41\xcd\x0f\xfc\x0b\x00\x41\x9c\x8e\x01\x41\xc8\x00\x41\xd3\x11\xfc\x0b\x00\x41\xe4\x8a\x03\x41\xc9\x00\x41\xf5\x18\xfc\x0b\x00\x41\x94\xd6\x00\x41\xca\x00\x41\xb0\x0f\xfc\x0b\x00\x41\xda\xfc\x00\x41\xcb\x00\x41\xaf\x0b\xfc\x0b\x00\x41\xde\xe2\x02\x41\xcc\x00\x41\x99\x09\xfc\x0b\x00\x41\xf9\xa6\x03\x41\xcd\x00\x41\xa0\x0c\xfc\x0b\x00\x41\xbb\x82\x02\x41\xce\x00\x41\xea\x0c\xfc\x0b\x00\x41\xe4\xdc\x03\x41\xcf\x00\x41\xd4\x19\xfc\x0b\x00\x41\x91\x94\x03\x41\xd0\x00\x41\xdf\x01\xfc\x0b\x00\x41\x89\x22\x41\xd1\x00\x41\xfb\x10\xfc\x0b\x00\x41\xaa\xc1\x03\x41\xd2\x00\x41\xaa\x0a\xfc\x0b\x00\x41\xac\xb3\x03\x41\xd3\x00\x41\xd8\x14\xfc\x0b\x00\x41\x9b\xbc\x01\x41\xd4\x00\x41\x95\x08\xfc\x0b\x00\x41\xaf\xd1\x02\x41\xd5\x00\x41\x99\x18\xfc\x0b\x00\x41\xb3\xfc\x01\x41\xd6\x00\x41\xec\x15\xfc\x0b\x00\x41\xe3\x1d\x41\xd7\x00\x41\xda\x0f\xfc\x0b\x00\x41\xc8\xac\x03\x41\xd8\x00\x41\x00\xfc\x0b\x00\x41\x95\x86\x03\x41\xd9\x00\x41\x95\x10\xfc\x0b\x00\x41\xbb\x9f\x01\x41\xda\x00\x41\xd0\x16\xfc\x0b\x00\x41\xa2\x88\x02\x41\xdb\x00\x41\xc0\x01\xfc\x0b\x00\x41\xba\xc9\x00\x41\xdc\x00\x41\x93\x11\xfc\x0b\x00\x41\xfd\xe0\x00\x41\xdd\x00\x41\x18\xfc\x0b\x00\x41\x8b\xee\x00\x41\xde\x00\x41\xc1\x04\xfc\x0b\x00\x41\x9a\xd8\x02\x41\xdf\x00\x41\xa9\x10\xfc\x0b\x00\x41\xff\x9e\x02\x41\xe0\x00\x41\xec\x1a\xfc\x0b\x00\x41\xf8\xb5\x01\x41\xe1\x00\x41\xcd\x15\xfc\x0b\x00\x41\xf8\x31\x41\xe2\x00\x41\xbe\x06\xfc\x0b\x00\x41\x9b\x84\x02\x41\xe3\x00\x41\x92\x0f\xfc\x0b\x00\x41\xb5\xab\x01\x41\xe4\x00\x41\xbe\x15\xfc\x0b\x00\x41\xce\xce\x03\x41\xe8\xa7\x03\x41\xb2\x10\xfc\x0a\x00\x00\x41\xb2\xec\x03\x41\xb8\xb2\x02\x41\xe6\x01\xfc\x0a\x00\x00\x41\xf9\x94\x03\x41\xcd\xb8\x01\x41\xfc\x11\xfc\x0a\x00\x00\x41\xb4\x34\x41\xbc\xbb\x01\x41\xff\x04\xfc\x0a\x00\x00\x41\xce\x36\x41\xf7\x84\x02\x41\xc9\x08\xfc\x0a\x00\x00\x41\xcb\x97\x01\x41\xec\xd0\x00\x41\xfd\x18\xfc\x0a\x00\x00\x41\xac\xd5\x01\x41\x86\xa9\x03\x41\xe4\x00\xfc\x0a\x00\x00\x41\xd5\xd4\x01\x41\xa2\xd5\x02\x41\xb5\x0d\xfc\x0a\x00\x00\x41\xf0\xd8\x03\x41\xb5\xc3\x00\x41\xf7\x00\xfc\x0a\x00\x00\x41\xbb\x2e\x41\x84\x12\x41\x92\x05\xfc\x0a\x00\x00\x41\xb3\x25\x41\xaf\x93\x03\x41\xdd\x11\xfc\x0a\x00\x00\x41\xc9\xe2\x00\x41\xfd\x95\x01\x41\xc1\x06\xfc\x0a\x00\x00\x41\xce\xdc\x00\x41\xa9\xeb\x02\x41\xe4\x19\xfc\x0a\x00\x00\x41\xf0\xd8\x00\x41\xd4\xdf\x02\x41\xe9\x11\xfc\x0a\x00\x00\x41\x8a\x8b\x02\x41\xa9\x34\x41\x8c\x14\xfc\x0a\x00\x00\x41\xc8\x26\x41\x9a\x0d\x41\xb0\x0a\xfc\x0a\x00\x00\x41\xbc\xed\x03\x41\xd5\x3b\x41\x86\x0d\xfc\x0a\x00\x00\x41\x98\xdc\x02\x41\xa8\x8f\x01\x41\x21\xfc\x0a\x00\x00\x41\x8e\xd7\x02\x41\xcc\xae\x01\x41\x93\x0b\xfc\x0a\x00\x00\x41\xad\xec\x02\x41\x9b\x85\x03\x41\x9a\x0b\xfc\x0a\x00\x00\x41\xc4\xf1\x03\x41\xb3\xc4\x00\x41\xc2\x06\xfc\x0a\x00\x00\x41\xcd\x85\x02\x41\xa3\x9d\x01\x41\xf5\x19\xfc\x0a\x00\x00\x41\xff\xbc\x02\x41\xad\xa8\x03\x41\x81\x19\xfc\x0a\x00\x00\x41\xd4\xc9\x01\x41\xf6\xce\x03\x41\x94\x13\xfc\x0a\x00\x00\x41\xde\x99\x01\x41\xb2\xbc\x03\x41\xda\x02\xfc\x0a\x00\x00\x41\xec\xfb\x00\x41\xca\x98\x02\x41\xfe\x12\xfc\x0a\x00\x00\x41\xb0\xdc\x00\x41\xf6\x95\x02\x41\xac\x02\xfc\x0a\x00\x00\x41\xa3\xd0\x03\x41\x85\xed\x00\x41\xd1\x18\xfc\x0a\x00\x00\x41\xfb\x8b\x02\x41\xb2\xd9\x03\x41\x81\x0a\xfc\x0a\x00\x00\x41\x84\xc6\x00\x41\xf4\xdf\x00\x41\xaf\x07\xfc\x0a\x00\x00\x41\x8b\x16\x41\xb9\xd1\x00\x41\xdf\x0e\xfc\x0a\x00\x00\x41\xba\xd1\x02\x41\x86\xd7\x02\x41\xe2\x05\xfc\x0a\x00\x00\x41\xbe\xec\x03\x41\x85\x94\x01\x41\xfa\x00\xfc\x0a\x00\x00\x41\xec\xbb\x01\x41\xd9\xdd\x02\x41\xdb\x0d\xfc\x0a\x00\x00\x41\xd0\xb0\x01\x41\xa3\xf3\x00\x41\xbe\x05\xfc\x0a\x00\x00\x41\x94\xd8\x00\x41\xd3\xcf\x01\x41\xa6\x0e\xfc\x0a\x00\x00\x41\xb4\xb4\x01\x41\xf7\x9f\x01\x41\xa8\x08\xfc\x0a\x00\x00\x41\xa0\xbf\x03\x41\xf2\xab\x03\x41\xc7\x14\xfc\x0a\x00\x00\x41\x94\xc7\x01\x41\x81\x08\x41\xa9\x18\xfc\x0a\x00\x00\x41\xb4\x83\x03\x41\xbc\xd9\x02\x41\xcf\x07\xfc\x0a\x00\x00\x41\xf8\xdc\x01\x41\xfa\xc5\x02\x41\xa0\x12\xfc\x0a\x00\x00\x41\xe9\xde\x03\x41\xe6\x01\x41\xb8\x16\xfc\x0a\x00\x00\x41\xd0\xaf\x01\x41\x9a\x9a\x03\x41\x95\x11\xfc\x0a\x00\x00\x41\xe9\xbc\x02\x41\xea\xca\x00\x41\xa6\x0f\xfc\x0a\x00\x00\x41\xcc\xe2\x01\x41\xfe\xa2\x01\x41\x8a\x11\xfc\x0a\x00\x00\x41\xa5\x9e\x03\x41\xb3\xd7\x02\x41\x8d\x08\xfc\x0a\x00\x00\x41\x84\xc7\x01\x41\xd3\x96\x02\x41\xf2\x0c\xfc\x0a\x00\x00\x41\x94\xc9\x03\x41\xfb\xe5\x02\x41\xc2\x0f\xfc\x0a\x00\x00\x41\x99\xab\x02\x41\x90\x2d\x41\xa3\x0f\xfc\x0a\x00\x00\x41\xd7\xde\x01\x41\xc4\xb0\x03\x41\xc0\x12\xfc\x0a\x00\x00\x41\x9b\xe9\x03\x41\xbc\x8d\x01\x41\xcc\x0a\xfc\x0a\x00\x00\x41\xe5\x87\x03\x41\xa5\xec\x00\x41\xfe\x02\xfc\x0a\x00\x00\x41\x88\x84\x01\x41\xf5\x9b\x02\x41\xec\x0e\xfc\x0a\x00\x00\x41\xe2\xf7\x02\x41\xde\xd8\x00\x41\xf7\x15\xfc\x0a\x00\x00\x41\xe0\xde\x01\x41\xaa\xbb\x02\x41\xc3\x02\xfc\x0a\x00\x00\x41\xb2\x95\x02\x41\xd0\xd9\x01\x41\x86\x0d\xfc\x0a\x00\x00\x41\xfa\xeb\x03\x41\xd4\xa0\x03\x41\xbd\x0a\xfc\x0a\x00\x00\x41\xb5\xee\x00\x41\xe8\xe9\x02\x41\x84\x05\xfc\x0a\x00\x00\x41\xe6\xe2\x01\x41\x82\x95\x01\x41\xf0\x03\xfc\x0a\x00\x00\x41\x98\xdf\x02\x41\xd9\xf3\x02\x41\xe0\x15\xfc\x0a\x00\x00\x41\x87\xb5\x02\x41\xf5\xdc\x02\x41\xc6\x0a\xfc\x0a\x00\x00\x41\xf0\xd0\x00\x41\xda\xe4\x01\x41\xc3\x0b\xfc\x0a\x00\x00\x41\xbf\xee\x02\x41\xe2\xe8\x02\x41\xbb\x0b\xfc\x0a\x00\x00\x41\xa9\x26\x41\xc4\xe0\x01\x41\xe7\x0e\xfc\x0a\x00\x00\x41\xfc\xa8\x02\x41\xa5\xbf\x03\x41\xd7\x0d\xfc\x0a\x00\x00\x41\xce\xce\x01\x41\xd7\xd4\x01\x41\xe7\x08\xfc\x0a\x00\x00\x41\xd3\xcb\x03\x41\xd1\xc0\x01\x41\xa7\x08\xfc\x0a\x00\x00\x41\xac\xdf\x03\x41\x86\xaf\x02\x41\xfe\x05\xfc\x0a\x00\x00\x41\x80\xd9\x02\x41\xec\x11\x41\xf0\x0b\xfc\x0a\x00\x00\x41\xe4\xff\x01\x41\x85\xf1\x02\x41\xc6\x17\xfc\x0a\x00\x00\x41\x8c\xd7\x00\x41\x8c\xa6\x01\x41\xf3\x07\xfc\x0a\x00\x00\x41\xf1\x3b\x41\xfc\xf6\x01\x41\xda\x17\xfc\x0a\x00\x00\x41\xfc\x8c\x01\x41\xbb\xe5\x00\x41\xf8\x19\xfc\x0a\x00\x00\x41\xda\xbf\x03\x41\xe1\xb4\x03\x41\xb4\x02\xfc\x0a\x00\x00\x41\xe3\xc0\x01\x41\xaf\x83\x01\x41\x83\x09\xfc\x0a\x00\x00\x41\xbc\x9b\x01\x41\x83\xcf\x00\x41\xd2\x05\xfc\x0a\x00\x00\x41\xe9\x16\x41\xaf\x2e\x41\xc2\x12\xfc\x0a\x00\x00\x41\xff\xfb\x01\x41\xaf\x87\x03\x41\xee\x16\xfc\x0a\x00\x00\x41\x96\xf6\x00\x41\x93\x87\x01\x41\xaf\x14\xfc\x0a\x00\x00\x41\x87\xe4\x02\x41\x9f\xde\x01\x41\xfd\x0f\xfc\x0a\x00\x00\x41\xed\xae\x03\x41\x91\x9a\x02\x41\xa4\x14\xfc\x0a\x00\x00\x41\xad\xde\x01\x41\x8d\xa7\x03\x41\x90\x09\xfc\x0a\x00\x00\x41\xcf\xf6\x02\x41\x89\xa1\x03\x41\xc1\x18\xfc\x0a\x00\x00\x41\xb6\xef\x01\x41\xe3\xe0\x02\x41\xd9\x14\xfc\x0a\x00\x00\x41\xc1\x27\x41\xc7\x21\x41\x34\xfc\x0a\x00\x00\x41\xa4\x34\x41\x83\xbd\x01\x41\xb9\x03\xfc\x0a\x00\x00\x41\xd8\x81\x02\x41\xed\xd3\x01\x41\xf5\x1a\xfc\x0a\x00\x00\x41\x92\xfe\x01\x41\xec\xcf\x03\x41\xe1\x15\xfc\x0a\x00\x00\x41\xb9\x8c\x02\x41\x82\xc6\x00\x41\xe6\x12\xfc\x0a\x00\x00\x41\xe5\x8b\x01\x41\x8a\xaa\x03\x41\xb5\x1a\xfc\x0a\x00\x00\x41\x9d\xb1\x01\x41\xf7\xd8\x02\x41\x88\x01\xfc\x0a\x00\x00\x41\xd1\xcd\x03\x41\xa5\x37\x41\x95\x08\xfc\x0a\x00\x00\x41\xc1\xcf\x02\x41\xf4\xad\x03\x41\xd5\x12\xfc\x0a\x00\x00\x41\x95\xdd\x02\x41\xaa\x9d\x01\x41\xed\x06\xfc\x0a\x00\x00\x41\xca\x9f\x02\x41\xec\xc4\x01\x41\xf7\x1a\xfc\x0a\x00\x00\x41\xae\xe5\x02\x41\x90\xf9\x01\x41\xd6\x06\xfc\x0a\x00\x00\x41\xac\xbd\x01\x41\xfa\xf8\x01\x41\xe1\x0a\xfc\x0a\x00\x00\x41\xf2\x87\x02\x41\xb4\x05\x41\xba\x0c\xfc\x0a\x00\x00\x41\xca\xd9\x03\x41\x99\x91\x01\x41\xab\x17\xfc\x0a\x00\x00\x41\xc2\x89\x03\x41\xb7\xc2\x02\x41\xfe\x0a\xfc\x0a\x00\x00\x0b\xa7\x80\x80\x80\x00\x00\x03\x40\x20\x00\x20\x01\x46\x04\x40\x41\x7f\x0f\x0b\x20\x00\x2d\x00\x00\x20\x02\x46\x04\x40\x20\x00\x41\x01\x6a\x21\x00\x0c\x01\x0b\x0b\x20\x00\x0f\x0b");

// memory_copy.wast:5115
run(() => call($33, "test", []));

// memory_copy.wast:5117
assert_return(() => call($33, "checkRange", [0, 124, 0]), -1);

// memory_copy.wast:5119
assert_return(() => call($33, "checkRange", [124, 1_517, 9]), -1);

// memory_copy.wast:5121
assert_return(() => call($33, "checkRange", [1_517, 2_132, 0]), -1);

// memory_copy.wast:5123
assert_return(() => call($33, "checkRange", [2_132, 2_827, 10]), -1);

// memory_copy.wast:5125
assert_return(() => call($33, "checkRange", [2_827, 2_921, 92]), -1);

// memory_copy.wast:5127
assert_return(() => call($33, "checkRange", [2_921, 3_538, 83]), -1);

// memory_copy.wast:5129
assert_return(() => call($33, "checkRange", [3_538, 3_786, 77]), -1);

// memory_copy.wast:5131
assert_return(() => call($33, "checkRange", [3_786, 4_042, 97]), -1);

// memory_copy.wast:5133
assert_return(() => call($33, "checkRange", [4_042, 4_651, 99]), -1);

// memory_copy.wast:5135
assert_return(() => call($33, "checkRange", [4_651, 5_057, 0]), -1);

// memory_copy.wast:5137
assert_return(() => call($33, "checkRange", [5_057, 5_109, 99]), -1);

// memory_copy.wast:5139
assert_return(() => call($33, "checkRange", [5_109, 5_291, 0]), -1);

// memory_copy.wast:5141
assert_return(() => call($33, "checkRange", [5_291, 5_524, 72]), -1);

// memory_copy.wast:5143
assert_return(() => call($33, "checkRange", [5_524, 5_691, 92]), -1);

// memory_copy.wast:5145
assert_return(() => call($33, "checkRange", [5_691, 6_552, 83]), -1);

// memory_copy.wast:5147
assert_return(() => call($33, "checkRange", [6_552, 7_133, 77]), -1);

// memory_copy.wast:5149
assert_return(() => call($33, "checkRange", [7_133, 7_665, 99]), -1);

// memory_copy.wast:5151
assert_return(() => call($33, "checkRange", [7_665, 8_314, 0]), -1);

// memory_copy.wast:5153
assert_return(() => call($33, "checkRange", [8_314, 8_360, 62]), -1);

// memory_copy.wast:5155
assert_return(() => call($33, "checkRange", [8_360, 8_793, 86]), -1);

// memory_copy.wast:5157
assert_return(() => call($33, "checkRange", [8_793, 8_979, 83]), -1);

// memory_copy.wast:5159
assert_return(() => call($33, "checkRange", [8_979, 9_373, 79]), -1);

// memory_copy.wast:5161
assert_return(() => call($33, "checkRange", [9_373, 9_518, 95]), -1);

// memory_copy.wast:5163
assert_return(() => call($33, "checkRange", [9_518, 9_934, 59]), -1);

// memory_copy.wast:5165
assert_return(() => call($33, "checkRange", [9_934, 10_087, 77]), -1);

// memory_copy.wast:5167
assert_return(() => call($33, "checkRange", [10_087, 10_206, 5]), -1);

// memory_copy.wast:5169
assert_return(() => call($33, "checkRange", [10_206, 10_230, 77]), -1);

// memory_copy.wast:5171
assert_return(() => call($33, "checkRange", [10_230, 10_249, 41]), -1);

// memory_copy.wast:5173
assert_return(() => call($33, "checkRange", [10_249, 11_148, 83]), -1);

// memory_copy.wast:5175
assert_return(() => call($33, "checkRange", [11_148, 11_356, 74]), -1);

// memory_copy.wast:5177
assert_return(() => call($33, "checkRange", [11_356, 11_380, 93]), -1);

// memory_copy.wast:5179
assert_return(() => call($33, "checkRange", [11_380, 11_939, 74]), -1);

// memory_copy.wast:5181
assert_return(() => call($33, "checkRange", [11_939, 12_159, 68]), -1);

// memory_copy.wast:5183
assert_return(() => call($33, "checkRange", [12_159, 12_575, 83]), -1);

// memory_copy.wast:5185
assert_return(() => call($33, "checkRange", [12_575, 12_969, 79]), -1);

// memory_copy.wast:5187
assert_return(() => call($33, "checkRange", [12_969, 13_114, 95]), -1);

// memory_copy.wast:5189
assert_return(() => call($33, "checkRange", [13_114, 14_133, 59]), -1);

// memory_copy.wast:5191
assert_return(() => call($33, "checkRange", [14_133, 14_404, 76]), -1);

// memory_copy.wast:5193
assert_return(() => call($33, "checkRange", [14_404, 14_428, 57]), -1);

// memory_copy.wast:5195
assert_return(() => call($33, "checkRange", [14_428, 14_458, 59]), -1);

// memory_copy.wast:5197
assert_return(() => call($33, "checkRange", [14_458, 14_580, 32]), -1);

// memory_copy.wast:5199
assert_return(() => call($33, "checkRange", [14_580, 14_777, 89]), -1);

// memory_copy.wast:5201
assert_return(() => call($33, "checkRange", [14_777, 15_124, 59]), -1);

// memory_copy.wast:5203
assert_return(() => call($33, "checkRange", [15_124, 15_126, 36]), -1);

// memory_copy.wast:5205
assert_return(() => call($33, "checkRange", [15_126, 15_192, 100]), -1);

// memory_copy.wast:5207
assert_return(() => call($33, "checkRange", [15_192, 15_871, 96]), -1);

// memory_copy.wast:5209
assert_return(() => call($33, "checkRange", [15_871, 15_998, 95]), -1);

// memory_copy.wast:5211
assert_return(() => call($33, "checkRange", [15_998, 17_017, 59]), -1);

// memory_copy.wast:5213
assert_return(() => call($33, "checkRange", [17_017, 17_288, 76]), -1);

// memory_copy.wast:5215
assert_return(() => call($33, "checkRange", [17_288, 17_312, 57]), -1);

// memory_copy.wast:5217
assert_return(() => call($33, "checkRange", [17_312, 17_342, 59]), -1);

// memory_copy.wast:5219
assert_return(() => call($33, "checkRange", [17_342, 17_464, 32]), -1);

// memory_copy.wast:5221
assert_return(() => call($33, "checkRange", [17_464, 17_661, 89]), -1);

// memory_copy.wast:5223
assert_return(() => call($33, "checkRange", [17_661, 17_727, 59]), -1);

// memory_copy.wast:5225
assert_return(() => call($33, "checkRange", [17_727, 17_733, 5]), -1);

// memory_copy.wast:5227
assert_return(() => call($33, "checkRange", [17_733, 17_893, 96]), -1);

// memory_copy.wast:5229
assert_return(() => call($33, "checkRange", [17_893, 18_553, 77]), -1);

// memory_copy.wast:5231
assert_return(() => call($33, "checkRange", [18_553, 18_744, 42]), -1);

// memory_copy.wast:5233
assert_return(() => call($33, "checkRange", [18_744, 18_801, 76]), -1);

// memory_copy.wast:5235
assert_return(() => call($33, "checkRange", [18_801, 18_825, 57]), -1);

// memory_copy.wast:5237
assert_return(() => call($33, "checkRange", [18_825, 18_876, 59]), -1);

// memory_copy.wast:5239
assert_return(() => call($33, "checkRange", [18_876, 18_885, 77]), -1);

// memory_copy.wast:5241
assert_return(() => call($33, "checkRange", [18_885, 18_904, 41]), -1);

// memory_copy.wast:5243
assert_return(() => call($33, "checkRange", [18_904, 19_567, 83]), -1);

// memory_copy.wast:5245
assert_return(() => call($33, "checkRange", [19_567, 20_403, 96]), -1);

// memory_copy.wast:5247
assert_return(() => call($33, "checkRange", [20_403, 21_274, 77]), -1);

// memory_copy.wast:5249
assert_return(() => call($33, "checkRange", [21_274, 21_364, 100]), -1);

// memory_copy.wast:5251
assert_return(() => call($33, "checkRange", [21_364, 21_468, 74]), -1);

// memory_copy.wast:5253
assert_return(() => call($33, "checkRange", [21_468, 21_492, 93]), -1);

// memory_copy.wast:5255
assert_return(() => call($33, "checkRange", [21_492, 22_051, 74]), -1);

// memory_copy.wast:5257
assert_return(() => call($33, "checkRange", [22_051, 22_480, 68]), -1);

// memory_copy.wast:5259
assert_return(() => call($33, "checkRange", [22_480, 22_685, 100]), -1);

// memory_copy.wast:5261
assert_return(() => call($33, "checkRange", [22_685, 22_694, 68]), -1);

// memory_copy.wast:5263
assert_return(() => call($33, "checkRange", [22_694, 22_821, 10]), -1);

// memory_copy.wast:5265
assert_return(() => call($33, "checkRange", [22_821, 22_869, 100]), -1);

// memory_copy.wast:5267
assert_return(() => call($33, "checkRange", [22_869, 24_107, 97]), -1);

// memory_copy.wast:5269
assert_return(() => call($33, "checkRange", [24_107, 24_111, 37]), -1);

// memory_copy.wast:5271
assert_return(() => call($33, "checkRange", [24_111, 24_236, 77]), -1);

// memory_copy.wast:5273
assert_return(() => call($33, "checkRange", [24_236, 24_348, 72]), -1);

// memory_copy.wast:5275
assert_return(() => call($33, "checkRange", [24_348, 24_515, 92]), -1);

// memory_copy.wast:5277
assert_return(() => call($33, "checkRange", [24_515, 24_900, 83]), -1);

// memory_copy.wast:5279
assert_return(() => call($33, "checkRange", [24_900, 25_136, 95]), -1);

// memory_copy.wast:5281
assert_return(() => call($33, "checkRange", [25_136, 25_182, 85]), -1);

// memory_copy.wast:5283
assert_return(() => call($33, "checkRange", [25_182, 25_426, 68]), -1);

// memory_copy.wast:5285
assert_return(() => call($33, "checkRange", [25_426, 25_613, 89]), -1);

// memory_copy.wast:5287
assert_return(() => call($33, "checkRange", [25_613, 25_830, 96]), -1);

// memory_copy.wast:5289
assert_return(() => call($33, "checkRange", [25_830, 26_446, 100]), -1);

// memory_copy.wast:5291
assert_return(() => call($33, "checkRange", [26_446, 26_517, 10]), -1);

// memory_copy.wast:5293
assert_return(() => call($33, "checkRange", [26_517, 27_468, 92]), -1);

// memory_copy.wast:5295
assert_return(() => call($33, "checkRange", [27_468, 27_503, 95]), -1);

// memory_copy.wast:5297
assert_return(() => call($33, "checkRange", [27_503, 27_573, 77]), -1);

// memory_copy.wast:5299
assert_return(() => call($33, "checkRange", [27_573, 28_245, 92]), -1);

// memory_copy.wast:5301
assert_return(() => call($33, "checkRange", [28_245, 28_280, 95]), -1);

// memory_copy.wast:5303
assert_return(() => call($33, "checkRange", [28_280, 29_502, 77]), -1);

// memory_copy.wast:5305
assert_return(() => call($33, "checkRange", [29_502, 29_629, 42]), -1);

// memory_copy.wast:5307
assert_return(() => call($33, "checkRange", [29_629, 30_387, 83]), -1);

// memory_copy.wast:5309
assert_return(() => call($33, "checkRange", [30_387, 30_646, 77]), -1);

// memory_copy.wast:5311
assert_return(() => call($33, "checkRange", [30_646, 31_066, 92]), -1);

// memory_copy.wast:5313
assert_return(() => call($33, "checkRange", [31_066, 31_131, 77]), -1);

// memory_copy.wast:5315
assert_return(() => call($33, "checkRange", [31_131, 31_322, 42]), -1);

// memory_copy.wast:5317
assert_return(() => call($33, "checkRange", [31_322, 31_379, 76]), -1);

// memory_copy.wast:5319
assert_return(() => call($33, "checkRange", [31_379, 31_403, 57]), -1);

// memory_copy.wast:5321
assert_return(() => call($33, "checkRange", [31_403, 31_454, 59]), -1);

// memory_copy.wast:5323
assert_return(() => call($33, "checkRange", [31_454, 31_463, 77]), -1);

// memory_copy.wast:5325
assert_return(() => call($33, "checkRange", [31_463, 31_482, 41]), -1);

// memory_copy.wast:5327
assert_return(() => call($33, "checkRange", [31_482, 31_649, 83]), -1);

// memory_copy.wast:5329
assert_return(() => call($33, "checkRange", [31_649, 31_978, 72]), -1);

// memory_copy.wast:5331
assert_return(() => call($33, "checkRange", [31_978, 32_145, 92]), -1);

// memory_copy.wast:5333
assert_return(() => call($33, "checkRange", [32_145, 32_530, 83]), -1);

// memory_copy.wast:5335
assert_return(() => call($33, "checkRange", [32_530, 32_766, 95]), -1);

// memory_copy.wast:5337
assert_return(() => call($33, "checkRange", [32_766, 32_812, 85]), -1);

// memory_copy.wast:5339
assert_return(() => call($33, "checkRange", [32_812, 33_056, 68]), -1);

// memory_copy.wast:5341
assert_return(() => call($33, "checkRange", [33_056, 33_660, 89]), -1);

// memory_copy.wast:5343
assert_return(() => call($33, "checkRange", [33_660, 33_752, 59]), -1);

// memory_copy.wast:5345
assert_return(() => call($33, "checkRange", [33_752, 33_775, 36]), -1);

// memory_copy.wast:5347
assert_return(() => call($33, "checkRange", [33_775, 33_778, 32]), -1);

// memory_copy.wast:5349
assert_return(() => call($33, "checkRange", [33_778, 34_603, 9]), -1);

// memory_copy.wast:5351
assert_return(() => call($33, "checkRange", [34_603, 35_218, 0]), -1);

// memory_copy.wast:5353
assert_return(() => call($33, "checkRange", [35_218, 35_372, 10]), -1);

// memory_copy.wast:5355
assert_return(() => call($33, "checkRange", [35_372, 35_486, 77]), -1);

// memory_copy.wast:5357
assert_return(() => call($33, "checkRange", [35_486, 35_605, 5]), -1);

// memory_copy.wast:5359
assert_return(() => call($33, "checkRange", [35_605, 35_629, 77]), -1);

// memory_copy.wast:5361
assert_return(() => call($33, "checkRange", [35_629, 35_648, 41]), -1);

// memory_copy.wast:5363
assert_return(() => call($33, "checkRange", [35_648, 36_547, 83]), -1);

// memory_copy.wast:5365
assert_return(() => call($33, "checkRange", [36_547, 36_755, 74]), -1);

// memory_copy.wast:5367
assert_return(() => call($33, "checkRange", [36_755, 36_767, 93]), -1);

// memory_copy.wast:5369
assert_return(() => call($33, "checkRange", [36_767, 36_810, 83]), -1);

// memory_copy.wast:5371
assert_return(() => call($33, "checkRange", [36_810, 36_839, 100]), -1);

// memory_copy.wast:5373
assert_return(() => call($33, "checkRange", [36_839, 37_444, 96]), -1);

// memory_copy.wast:5375
assert_return(() => call($33, "checkRange", [37_444, 38_060, 100]), -1);

// memory_copy.wast:5377
assert_return(() => call($33, "checkRange", [38_060, 38_131, 10]), -1);

// memory_copy.wast:5379
assert_return(() => call($33, "checkRange", [38_131, 39_082, 92]), -1);

// memory_copy.wast:5381
assert_return(() => call($33, "checkRange", [39_082, 39_117, 95]), -1);

// memory_copy.wast:5383
assert_return(() => call($33, "checkRange", [39_117, 39_187, 77]), -1);

// memory_copy.wast:5385
assert_return(() => call($33, "checkRange", [39_187, 39_859, 92]), -1);

// memory_copy.wast:5387
assert_return(() => call($33, "checkRange", [39_859, 39_894, 95]), -1);

// memory_copy.wast:5389
assert_return(() => call($33, "checkRange", [39_894, 40_257, 77]), -1);

// memory_copy.wast:5391
assert_return(() => call($33, "checkRange", [40_257, 40_344, 89]), -1);

// memory_copy.wast:5393
assert_return(() => call($33, "checkRange", [40_344, 40_371, 59]), -1);

// memory_copy.wast:5395
assert_return(() => call($33, "checkRange", [40_371, 40_804, 77]), -1);

// memory_copy.wast:5397
assert_return(() => call($33, "checkRange", [40_804, 40_909, 5]), -1);

// memory_copy.wast:5399
assert_return(() => call($33, "checkRange", [40_909, 42_259, 92]), -1);

// memory_copy.wast:5401
assert_return(() => call($33, "checkRange", [42_259, 42_511, 77]), -1);

// memory_copy.wast:5403
assert_return(() => call($33, "checkRange", [42_511, 42_945, 83]), -1);

// memory_copy.wast:5405
assert_return(() => call($33, "checkRange", [42_945, 43_115, 77]), -1);

// memory_copy.wast:5407
assert_return(() => call($33, "checkRange", [43_115, 43_306, 42]), -1);

// memory_copy.wast:5409
assert_return(() => call($33, "checkRange", [43_306, 43_363, 76]), -1);

// memory_copy.wast:5411
assert_return(() => call($33, "checkRange", [43_363, 43_387, 57]), -1);

// memory_copy.wast:5413
assert_return(() => call($33, "checkRange", [43_387, 43_438, 59]), -1);

// memory_copy.wast:5415
assert_return(() => call($33, "checkRange", [43_438, 43_447, 77]), -1);

// memory_copy.wast:5417
assert_return(() => call($33, "checkRange", [43_447, 43_466, 41]), -1);

// memory_copy.wast:5419
assert_return(() => call($33, "checkRange", [43_466, 44_129, 83]), -1);

// memory_copy.wast:5421
assert_return(() => call($33, "checkRange", [44_129, 44_958, 96]), -1);

// memory_copy.wast:5423
assert_return(() => call($33, "checkRange", [44_958, 45_570, 77]), -1);

// memory_copy.wast:5425
assert_return(() => call($33, "checkRange", [45_570, 45_575, 92]), -1);

// memory_copy.wast:5427
assert_return(() => call($33, "checkRange", [45_575, 45_640, 77]), -1);

// memory_copy.wast:5429
assert_return(() => call($33, "checkRange", [45_640, 45_742, 42]), -1);

// memory_copy.wast:5431
assert_return(() => call($33, "checkRange", [45_742, 45_832, 72]), -1);

// memory_copy.wast:5433
assert_return(() => call($33, "checkRange", [45_832, 45_999, 92]), -1);

// memory_copy.wast:5435
assert_return(() => call($33, "checkRange", [45_999, 46_384, 83]), -1);

// memory_copy.wast:5437
assert_return(() => call($33, "checkRange", [46_384, 46_596, 95]), -1);

// memory_copy.wast:5439
assert_return(() => call($33, "checkRange", [46_596, 46_654, 92]), -1);

// memory_copy.wast:5441
assert_return(() => call($33, "checkRange", [46_654, 47_515, 83]), -1);

// memory_copy.wast:5443
assert_return(() => call($33, "checkRange", [47_515, 47_620, 77]), -1);

// memory_copy.wast:5445
assert_return(() => call($33, "checkRange", [47_620, 47_817, 79]), -1);

// memory_copy.wast:5447
assert_return(() => call($33, "checkRange", [47_817, 47_951, 95]), -1);

// memory_copy.wast:5449
assert_return(() => call($33, "checkRange", [47_951, 48_632, 100]), -1);

// memory_copy.wast:5451
assert_return(() => call($33, "checkRange", [48_632, 48_699, 97]), -1);

// memory_copy.wast:5453
assert_return(() => call($33, "checkRange", [48_699, 48_703, 37]), -1);

// memory_copy.wast:5455
assert_return(() => call($33, "checkRange", [48_703, 49_764, 77]), -1);

// memory_copy.wast:5457
assert_return(() => call($33, "checkRange", [49_764, 49_955, 42]), -1);

// memory_copy.wast:5459
assert_return(() => call($33, "checkRange", [49_955, 50_012, 76]), -1);

// memory_copy.wast:5461
assert_return(() => call($33, "checkRange", [50_012, 50_036, 57]), -1);

// memory_copy.wast:5463
assert_return(() => call($33, "checkRange", [50_036, 50_087, 59]), -1);

// memory_copy.wast:5465
assert_return(() => call($33, "checkRange", [50_087, 50_096, 77]), -1);

// memory_copy.wast:5467
assert_return(() => call($33, "checkRange", [50_096, 50_115, 41]), -1);

// memory_copy.wast:5469
assert_return(() => call($33, "checkRange", [50_115, 50_370, 83]), -1);

// memory_copy.wast:5471
assert_return(() => call($33, "checkRange", [50_370, 51_358, 92]), -1);

// memory_copy.wast:5473
assert_return(() => call($33, "checkRange", [51_358, 51_610, 77]), -1);

// memory_copy.wast:5475
assert_return(() => call($33, "checkRange", [51_610, 51_776, 83]), -1);

// memory_copy.wast:5477
assert_return(() => call($33, "checkRange", [51_776, 51_833, 89]), -1);

// memory_copy.wast:5479
assert_return(() => call($33, "checkRange", [51_833, 52_895, 100]), -1);

// memory_copy.wast:5481
assert_return(() => call($33, "checkRange", [52_895, 53_029, 97]), -1);

// memory_copy.wast:5483
assert_return(() => call($33, "checkRange", [53_029, 53_244, 68]), -1);

// memory_copy.wast:5485
assert_return(() => call($33, "checkRange", [53_244, 54_066, 100]), -1);

// memory_copy.wast:5487
assert_return(() => call($33, "checkRange", [54_066, 54_133, 97]), -1);

// memory_copy.wast:5489
assert_return(() => call($33, "checkRange", [54_133, 54_137, 37]), -1);

// memory_copy.wast:5491
assert_return(() => call($33, "checkRange", [54_137, 55_198, 77]), -1);

// memory_copy.wast:5493
assert_return(() => call($33, "checkRange", [55_198, 55_389, 42]), -1);

// memory_copy.wast:5495
assert_return(() => call($33, "checkRange", [55_389, 55_446, 76]), -1);

// memory_copy.wast:5497
assert_return(() => call($33, "checkRange", [55_446, 55_470, 57]), -1);

// memory_copy.wast:5499
assert_return(() => call($33, "checkRange", [55_470, 55_521, 59]), -1);

// memory_copy.wast:5501
assert_return(() => call($33, "checkRange", [55_521, 55_530, 77]), -1);

// memory_copy.wast:5503
assert_return(() => call($33, "checkRange", [55_530, 55_549, 41]), -1);

// memory_copy.wast:5505
assert_return(() => call($33, "checkRange", [55_549, 56_212, 83]), -1);

// memory_copy.wast:5507
assert_return(() => call($33, "checkRange", [56_212, 57_048, 96]), -1);

// memory_copy.wast:5509
assert_return(() => call($33, "checkRange", [57_048, 58_183, 77]), -1);

// memory_copy.wast:5511
assert_return(() => call($33, "checkRange", [58_183, 58_202, 41]), -1);

// memory_copy.wast:5513
assert_return(() => call($33, "checkRange", [58_202, 58_516, 83]), -1);

// memory_copy.wast:5515
assert_return(() => call($33, "checkRange", [58_516, 58_835, 95]), -1);

// memory_copy.wast:5517
assert_return(() => call($33, "checkRange", [58_835, 58_855, 77]), -1);

// memory_copy.wast:5519
assert_return(() => call($33, "checkRange", [58_855, 59_089, 95]), -1);

// memory_copy.wast:5521
assert_return(() => call($33, "checkRange", [59_089, 59_145, 77]), -1);

// memory_copy.wast:5523
assert_return(() => call($33, "checkRange", [59_145, 59_677, 99]), -1);

// memory_copy.wast:5525
assert_return(() => call($33, "checkRange", [59_677, 60_134, 0]), -1);

// memory_copy.wast:5527
assert_return(() => call($33, "checkRange", [60_134, 60_502, 89]), -1);

// memory_copy.wast:5529
assert_return(() => call($33, "checkRange", [60_502, 60_594, 59]), -1);

// memory_copy.wast:5531
assert_return(() => call($33, "checkRange", [60_594, 60_617, 36]), -1);

// memory_copy.wast:5533
assert_return(() => call($33, "checkRange", [60_617, 60_618, 32]), -1);

// memory_copy.wast:5535
assert_return(() => call($33, "checkRange", [60_618, 60_777, 42]), -1);

// memory_copy.wast:5537
assert_return(() => call($33, "checkRange", [60_777, 60_834, 76]), -1);

// memory_copy.wast:5539
assert_return(() => call($33, "checkRange", [60_834, 60_858, 57]), -1);

// memory_copy.wast:5541
assert_return(() => call($33, "checkRange", [60_858, 60_909, 59]), -1);

// memory_copy.wast:5543
assert_return(() => call($33, "checkRange", [60_909, 60_918, 77]), -1);

// memory_copy.wast:5545
assert_return(() => call($33, "checkRange", [60_918, 60_937, 41]), -1);

// memory_copy.wast:5547
assert_return(() => call($33, "checkRange", [60_937, 61_600, 83]), -1);

// memory_copy.wast:5549
assert_return(() => call($33, "checkRange", [61_600, 62_436, 96]), -1);

// memory_copy.wast:5551
assert_return(() => call($33, "checkRange", [62_436, 63_307, 77]), -1);

// memory_copy.wast:5553
assert_return(() => call($33, "checkRange", [63_307, 63_397, 100]), -1);

// memory_copy.wast:5555
assert_return(() => call($33, "checkRange", [63_397, 63_501, 74]), -1);

// memory_copy.wast:5557
assert_return(() => call($33, "checkRange", [63_501, 63_525, 93]), -1);

// memory_copy.wast:5559
assert_return(() => call($33, "checkRange", [63_525, 63_605, 74]), -1);

// memory_copy.wast:5561
assert_return(() => call($33, "checkRange", [63_605, 63_704, 100]), -1);

// memory_copy.wast:5563
assert_return(() => call($33, "checkRange", [63_704, 63_771, 97]), -1);

// memory_copy.wast:5565
assert_return(() => call($33, "checkRange", [63_771, 63_775, 37]), -1);

// memory_copy.wast:5567
assert_return(() => call($33, "checkRange", [63_775, 64_311, 77]), -1);

// memory_copy.wast:5569
assert_return(() => call($33, "checkRange", [64_311, 64_331, 26]), -1);

// memory_copy.wast:5571
assert_return(() => call($33, "checkRange", [64_331, 64_518, 92]), -1);

// memory_copy.wast:5573
assert_return(() => call($33, "checkRange", [64_518, 64_827, 11]), -1);

// memory_copy.wast:5575
assert_return(() => call($33, "checkRange", [64_827, 64_834, 26]), -1);

// memory_copy.wast:5577
assert_return(() => call($33, "checkRange", [64_834, 65_536, 0]), -1);
reinitializeRegistry();
})();
