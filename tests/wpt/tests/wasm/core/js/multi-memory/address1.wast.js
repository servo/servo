(function address1_wast_js() {

// address1.wast:3
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8a\x80\x80\x80\x00\x02\x60\x01\x7f\x01\x7e\x60\x01\x7f\x00\x03\xab\x80\x80\x80\x00\x2a\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x01\x01\x01\x01\x01\x01\x05\x8b\x80\x80\x80\x00\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x07\xd9\x83\x80\x80\x00\x2a\x08\x38\x75\x5f\x67\x6f\x6f\x64\x31\x00\x00\x08\x38\x75\x5f\x67\x6f\x6f\x64\x32\x00\x01\x08\x38\x75\x5f\x67\x6f\x6f\x64\x33\x00\x02\x08\x38\x75\x5f\x67\x6f\x6f\x64\x34\x00\x03\x08\x38\x75\x5f\x67\x6f\x6f\x64\x35\x00\x04\x08\x38\x73\x5f\x67\x6f\x6f\x64\x31\x00\x05\x08\x38\x73\x5f\x67\x6f\x6f\x64\x32\x00\x06\x08\x38\x73\x5f\x67\x6f\x6f\x64\x33\x00\x07\x08\x38\x73\x5f\x67\x6f\x6f\x64\x34\x00\x08\x08\x38\x73\x5f\x67\x6f\x6f\x64\x35\x00\x09\x09\x31\x36\x75\x5f\x67\x6f\x6f\x64\x31\x00\x0a\x09\x31\x36\x75\x5f\x67\x6f\x6f\x64\x32\x00\x0b\x09\x31\x36\x75\x5f\x67\x6f\x6f\x64\x33\x00\x0c\x09\x31\x36\x75\x5f\x67\x6f\x6f\x64\x34\x00\x0d\x09\x31\x36\x75\x5f\x67\x6f\x6f\x64\x35\x00\x0e\x09\x31\x36\x73\x5f\x67\x6f\x6f\x64\x31\x00\x0f\x09\x31\x36\x73\x5f\x67\x6f\x6f\x64\x32\x00\x10\x09\x31\x36\x73\x5f\x67\x6f\x6f\x64\x33\x00\x11\x09\x31\x36\x73\x5f\x67\x6f\x6f\x64\x34\x00\x12\x09\x31\x36\x73\x5f\x67\x6f\x6f\x64\x35\x00\x13\x09\x33\x32\x75\x5f\x67\x6f\x6f\x64\x31\x00\x14\x09\x33\x32\x75\x5f\x67\x6f\x6f\x64\x32\x00\x15\x09\x33\x32\x75\x5f\x67\x6f\x6f\x64\x33\x00\x16\x09\x33\x32\x75\x5f\x67\x6f\x6f\x64\x34\x00\x17\x09\x33\x32\x75\x5f\x67\x6f\x6f\x64\x35\x00\x18\x09\x33\x32\x73\x5f\x67\x6f\x6f\x64\x31\x00\x19\x09\x33\x32\x73\x5f\x67\x6f\x6f\x64\x32\x00\x1a\x09\x33\x32\x73\x5f\x67\x6f\x6f\x64\x33\x00\x1b\x09\x33\x32\x73\x5f\x67\x6f\x6f\x64\x34\x00\x1c\x09\x33\x32\x73\x5f\x67\x6f\x6f\x64\x35\x00\x1d\x08\x36\x34\x5f\x67\x6f\x6f\x64\x31\x00\x1e\x08\x36\x34\x5f\x67\x6f\x6f\x64\x32\x00\x1f\x08\x36\x34\x5f\x67\x6f\x6f\x64\x33\x00\x20\x08\x36\x34\x5f\x67\x6f\x6f\x64\x34\x00\x21\x08\x36\x34\x5f\x67\x6f\x6f\x64\x35\x00\x22\x06\x38\x75\x5f\x62\x61\x64\x00\x23\x06\x38\x73\x5f\x62\x61\x64\x00\x24\x07\x31\x36\x75\x5f\x62\x61\x64\x00\x25\x07\x31\x36\x73\x5f\x62\x61\x64\x00\x26\x07\x33\x32\x75\x5f\x62\x61\x64\x00\x27\x07\x33\x32\x73\x5f\x62\x61\x64\x00\x28\x06\x36\x34\x5f\x62\x61\x64\x00\x29\x0a\xc6\x84\x80\x80\x00\x2a\x88\x80\x80\x80\x00\x00\x20\x00\x31\x40\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x31\x40\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x31\x40\x04\x01\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x31\x40\x04\x02\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x31\x40\x04\x19\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x30\x40\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x30\x40\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x30\x40\x04\x01\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x30\x40\x04\x02\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x30\x40\x04\x19\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x33\x41\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x33\x40\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x33\x40\x04\x01\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x33\x41\x04\x02\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x33\x41\x04\x19\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x32\x41\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x32\x40\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x32\x40\x04\x01\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x32\x41\x04\x02\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x32\x41\x04\x19\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x35\x42\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x35\x40\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x35\x40\x04\x01\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x35\x41\x04\x02\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x35\x42\x04\x19\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x34\x42\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x34\x40\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x34\x40\x04\x01\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x34\x41\x04\x02\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x34\x42\x04\x19\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x29\x43\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x29\x40\x04\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x29\x40\x04\x01\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x29\x41\x04\x02\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x29\x43\x04\x19\x0b\x8d\x80\x80\x80\x00\x00\x20\x00\x31\x40\x04\xff\xff\xff\xff\x0f\x1a\x0b\x8d\x80\x80\x80\x00\x00\x20\x00\x30\x40\x04\xff\xff\xff\xff\x0f\x1a\x0b\x8d\x80\x80\x80\x00\x00\x20\x00\x33\x41\x04\xff\xff\xff\xff\x0f\x1a\x0b\x8d\x80\x80\x80\x00\x00\x20\x00\x32\x41\x04\xff\xff\xff\xff\x0f\x1a\x0b\x8d\x80\x80\x80\x00\x00\x20\x00\x35\x42\x04\xff\xff\xff\xff\x0f\x1a\x0b\x8d\x80\x80\x80\x00\x00\x20\x00\x34\x42\x04\xff\xff\xff\xff\x0f\x1a\x0b\x8d\x80\x80\x80\x00\x00\x20\x00\x29\x43\x04\xff\xff\xff\xff\x0f\x1a\x0b\x0b\xa1\x80\x80\x80\x00\x01\x02\x04\x41\x00\x0b\x1a\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6a\x6b\x6c\x6d\x6e\x6f\x70\x71\x72\x73\x74\x75\x76\x77\x78\x79\x7a", "address1.wast:3");

// address1.wast:3
let $1 = instance($$1);

// address1.wast:146
assert_return(() => call($1, "8u_good1", [0]), "address1.wast:146", 97n);

// address1.wast:147
assert_return(() => call($1, "8u_good2", [0]), "address1.wast:147", 97n);

// address1.wast:148
assert_return(() => call($1, "8u_good3", [0]), "address1.wast:148", 98n);

// address1.wast:149
assert_return(() => call($1, "8u_good4", [0]), "address1.wast:149", 99n);

// address1.wast:150
assert_return(() => call($1, "8u_good5", [0]), "address1.wast:150", 122n);

// address1.wast:152
assert_return(() => call($1, "8s_good1", [0]), "address1.wast:152", 97n);

// address1.wast:153
assert_return(() => call($1, "8s_good2", [0]), "address1.wast:153", 97n);

// address1.wast:154
assert_return(() => call($1, "8s_good3", [0]), "address1.wast:154", 98n);

// address1.wast:155
assert_return(() => call($1, "8s_good4", [0]), "address1.wast:155", 99n);

// address1.wast:156
assert_return(() => call($1, "8s_good5", [0]), "address1.wast:156", 122n);

// address1.wast:158
assert_return(() => call($1, "16u_good1", [0]), "address1.wast:158", 25_185n);

// address1.wast:159
assert_return(() => call($1, "16u_good2", [0]), "address1.wast:159", 25_185n);

// address1.wast:160
assert_return(() => call($1, "16u_good3", [0]), "address1.wast:160", 25_442n);

// address1.wast:161
assert_return(() => call($1, "16u_good4", [0]), "address1.wast:161", 25_699n);

// address1.wast:162
assert_return(() => call($1, "16u_good5", [0]), "address1.wast:162", 122n);

// address1.wast:164
assert_return(() => call($1, "16s_good1", [0]), "address1.wast:164", 25_185n);

// address1.wast:165
assert_return(() => call($1, "16s_good2", [0]), "address1.wast:165", 25_185n);

// address1.wast:166
assert_return(() => call($1, "16s_good3", [0]), "address1.wast:166", 25_442n);

// address1.wast:167
assert_return(() => call($1, "16s_good4", [0]), "address1.wast:167", 25_699n);

// address1.wast:168
assert_return(() => call($1, "16s_good5", [0]), "address1.wast:168", 122n);

// address1.wast:170
assert_return(() => call($1, "32u_good1", [0]), "address1.wast:170", 1_684_234_849n);

// address1.wast:171
assert_return(() => call($1, "32u_good2", [0]), "address1.wast:171", 1_684_234_849n);

// address1.wast:172
assert_return(() => call($1, "32u_good3", [0]), "address1.wast:172", 1_701_077_858n);

// address1.wast:173
assert_return(() => call($1, "32u_good4", [0]), "address1.wast:173", 1_717_920_867n);

// address1.wast:174
assert_return(() => call($1, "32u_good5", [0]), "address1.wast:174", 122n);

// address1.wast:176
assert_return(() => call($1, "32s_good1", [0]), "address1.wast:176", 1_684_234_849n);

// address1.wast:177
assert_return(() => call($1, "32s_good2", [0]), "address1.wast:177", 1_684_234_849n);

// address1.wast:178
assert_return(() => call($1, "32s_good3", [0]), "address1.wast:178", 1_701_077_858n);

// address1.wast:179
assert_return(() => call($1, "32s_good4", [0]), "address1.wast:179", 1_717_920_867n);

// address1.wast:180
assert_return(() => call($1, "32s_good5", [0]), "address1.wast:180", 122n);

// address1.wast:182
assert_return(() => call($1, "64_good1", [0]), "address1.wast:182", 7_523_094_288_207_667_809n);

// address1.wast:183
assert_return(() => call($1, "64_good2", [0]), "address1.wast:183", 7_523_094_288_207_667_809n);

// address1.wast:184
assert_return(() => call($1, "64_good3", [0]), "address1.wast:184", 7_595_434_461_045_744_482n);

// address1.wast:185
assert_return(() => call($1, "64_good4", [0]), "address1.wast:185", 7_667_774_633_883_821_155n);

// address1.wast:186
assert_return(() => call($1, "64_good5", [0]), "address1.wast:186", 122n);

// address1.wast:188
assert_return(() => call($1, "8u_good1", [65_503]), "address1.wast:188", 0n);

// address1.wast:189
assert_return(() => call($1, "8u_good2", [65_503]), "address1.wast:189", 0n);

// address1.wast:190
assert_return(() => call($1, "8u_good3", [65_503]), "address1.wast:190", 0n);

// address1.wast:191
assert_return(() => call($1, "8u_good4", [65_503]), "address1.wast:191", 0n);

// address1.wast:192
assert_return(() => call($1, "8u_good5", [65_503]), "address1.wast:192", 0n);

// address1.wast:194
assert_return(() => call($1, "8s_good1", [65_503]), "address1.wast:194", 0n);

// address1.wast:195
assert_return(() => call($1, "8s_good2", [65_503]), "address1.wast:195", 0n);

// address1.wast:196
assert_return(() => call($1, "8s_good3", [65_503]), "address1.wast:196", 0n);

// address1.wast:197
assert_return(() => call($1, "8s_good4", [65_503]), "address1.wast:197", 0n);

// address1.wast:198
assert_return(() => call($1, "8s_good5", [65_503]), "address1.wast:198", 0n);

// address1.wast:200
assert_return(() => call($1, "16u_good1", [65_503]), "address1.wast:200", 0n);

// address1.wast:201
assert_return(() => call($1, "16u_good2", [65_503]), "address1.wast:201", 0n);

// address1.wast:202
assert_return(() => call($1, "16u_good3", [65_503]), "address1.wast:202", 0n);

// address1.wast:203
assert_return(() => call($1, "16u_good4", [65_503]), "address1.wast:203", 0n);

// address1.wast:204
assert_return(() => call($1, "16u_good5", [65_503]), "address1.wast:204", 0n);

// address1.wast:206
assert_return(() => call($1, "16s_good1", [65_503]), "address1.wast:206", 0n);

// address1.wast:207
assert_return(() => call($1, "16s_good2", [65_503]), "address1.wast:207", 0n);

// address1.wast:208
assert_return(() => call($1, "16s_good3", [65_503]), "address1.wast:208", 0n);

// address1.wast:209
assert_return(() => call($1, "16s_good4", [65_503]), "address1.wast:209", 0n);

// address1.wast:210
assert_return(() => call($1, "16s_good5", [65_503]), "address1.wast:210", 0n);

// address1.wast:212
assert_return(() => call($1, "32u_good1", [65_503]), "address1.wast:212", 0n);

// address1.wast:213
assert_return(() => call($1, "32u_good2", [65_503]), "address1.wast:213", 0n);

// address1.wast:214
assert_return(() => call($1, "32u_good3", [65_503]), "address1.wast:214", 0n);

// address1.wast:215
assert_return(() => call($1, "32u_good4", [65_503]), "address1.wast:215", 0n);

// address1.wast:216
assert_return(() => call($1, "32u_good5", [65_503]), "address1.wast:216", 0n);

// address1.wast:218
assert_return(() => call($1, "32s_good1", [65_503]), "address1.wast:218", 0n);

// address1.wast:219
assert_return(() => call($1, "32s_good2", [65_503]), "address1.wast:219", 0n);

// address1.wast:220
assert_return(() => call($1, "32s_good3", [65_503]), "address1.wast:220", 0n);

// address1.wast:221
assert_return(() => call($1, "32s_good4", [65_503]), "address1.wast:221", 0n);

// address1.wast:222
assert_return(() => call($1, "32s_good5", [65_503]), "address1.wast:222", 0n);

// address1.wast:224
assert_return(() => call($1, "64_good1", [65_503]), "address1.wast:224", 0n);

// address1.wast:225
assert_return(() => call($1, "64_good2", [65_503]), "address1.wast:225", 0n);

// address1.wast:226
assert_return(() => call($1, "64_good3", [65_503]), "address1.wast:226", 0n);

// address1.wast:227
assert_return(() => call($1, "64_good4", [65_503]), "address1.wast:227", 0n);

// address1.wast:228
assert_return(() => call($1, "64_good5", [65_503]), "address1.wast:228", 0n);

// address1.wast:230
assert_return(() => call($1, "8u_good1", [65_504]), "address1.wast:230", 0n);

// address1.wast:231
assert_return(() => call($1, "8u_good2", [65_504]), "address1.wast:231", 0n);

// address1.wast:232
assert_return(() => call($1, "8u_good3", [65_504]), "address1.wast:232", 0n);

// address1.wast:233
assert_return(() => call($1, "8u_good4", [65_504]), "address1.wast:233", 0n);

// address1.wast:234
assert_return(() => call($1, "8u_good5", [65_504]), "address1.wast:234", 0n);

// address1.wast:236
assert_return(() => call($1, "8s_good1", [65_504]), "address1.wast:236", 0n);

// address1.wast:237
assert_return(() => call($1, "8s_good2", [65_504]), "address1.wast:237", 0n);

// address1.wast:238
assert_return(() => call($1, "8s_good3", [65_504]), "address1.wast:238", 0n);

// address1.wast:239
assert_return(() => call($1, "8s_good4", [65_504]), "address1.wast:239", 0n);

// address1.wast:240
assert_return(() => call($1, "8s_good5", [65_504]), "address1.wast:240", 0n);

// address1.wast:242
assert_return(() => call($1, "16u_good1", [65_504]), "address1.wast:242", 0n);

// address1.wast:243
assert_return(() => call($1, "16u_good2", [65_504]), "address1.wast:243", 0n);

// address1.wast:244
assert_return(() => call($1, "16u_good3", [65_504]), "address1.wast:244", 0n);

// address1.wast:245
assert_return(() => call($1, "16u_good4", [65_504]), "address1.wast:245", 0n);

// address1.wast:246
assert_return(() => call($1, "16u_good5", [65_504]), "address1.wast:246", 0n);

// address1.wast:248
assert_return(() => call($1, "16s_good1", [65_504]), "address1.wast:248", 0n);

// address1.wast:249
assert_return(() => call($1, "16s_good2", [65_504]), "address1.wast:249", 0n);

// address1.wast:250
assert_return(() => call($1, "16s_good3", [65_504]), "address1.wast:250", 0n);

// address1.wast:251
assert_return(() => call($1, "16s_good4", [65_504]), "address1.wast:251", 0n);

// address1.wast:252
assert_return(() => call($1, "16s_good5", [65_504]), "address1.wast:252", 0n);

// address1.wast:254
assert_return(() => call($1, "32u_good1", [65_504]), "address1.wast:254", 0n);

// address1.wast:255
assert_return(() => call($1, "32u_good2", [65_504]), "address1.wast:255", 0n);

// address1.wast:256
assert_return(() => call($1, "32u_good3", [65_504]), "address1.wast:256", 0n);

// address1.wast:257
assert_return(() => call($1, "32u_good4", [65_504]), "address1.wast:257", 0n);

// address1.wast:258
assert_return(() => call($1, "32u_good5", [65_504]), "address1.wast:258", 0n);

// address1.wast:260
assert_return(() => call($1, "32s_good1", [65_504]), "address1.wast:260", 0n);

// address1.wast:261
assert_return(() => call($1, "32s_good2", [65_504]), "address1.wast:261", 0n);

// address1.wast:262
assert_return(() => call($1, "32s_good3", [65_504]), "address1.wast:262", 0n);

// address1.wast:263
assert_return(() => call($1, "32s_good4", [65_504]), "address1.wast:263", 0n);

// address1.wast:264
assert_return(() => call($1, "32s_good5", [65_504]), "address1.wast:264", 0n);

// address1.wast:266
assert_return(() => call($1, "64_good1", [65_504]), "address1.wast:266", 0n);

// address1.wast:267
assert_return(() => call($1, "64_good2", [65_504]), "address1.wast:267", 0n);

// address1.wast:268
assert_return(() => call($1, "64_good3", [65_504]), "address1.wast:268", 0n);

// address1.wast:269
assert_return(() => call($1, "64_good4", [65_504]), "address1.wast:269", 0n);

// address1.wast:270
assert_trap(() => call($1, "64_good5", [65_504]), "address1.wast:270");

// address1.wast:272
assert_trap(() => call($1, "8u_good3", [-1]), "address1.wast:272");

// address1.wast:273
assert_trap(() => call($1, "8s_good3", [-1]), "address1.wast:273");

// address1.wast:274
assert_trap(() => call($1, "16u_good3", [-1]), "address1.wast:274");

// address1.wast:275
assert_trap(() => call($1, "16s_good3", [-1]), "address1.wast:275");

// address1.wast:276
assert_trap(() => call($1, "32u_good3", [-1]), "address1.wast:276");

// address1.wast:277
assert_trap(() => call($1, "32s_good3", [-1]), "address1.wast:277");

// address1.wast:278
assert_trap(() => call($1, "64_good3", [-1]), "address1.wast:278");

// address1.wast:280
assert_trap(() => call($1, "8u_bad", [0]), "address1.wast:280");

// address1.wast:281
assert_trap(() => call($1, "8s_bad", [0]), "address1.wast:281");

// address1.wast:282
assert_trap(() => call($1, "16u_bad", [0]), "address1.wast:282");

// address1.wast:283
assert_trap(() => call($1, "16s_bad", [0]), "address1.wast:283");

// address1.wast:284
assert_trap(() => call($1, "32u_bad", [0]), "address1.wast:284");

// address1.wast:285
assert_trap(() => call($1, "32s_bad", [0]), "address1.wast:285");

// address1.wast:286
assert_trap(() => call($1, "64_bad", [0]), "address1.wast:286");

// address1.wast:288
assert_trap(() => call($1, "8u_bad", [1]), "address1.wast:288");

// address1.wast:289
assert_trap(() => call($1, "8s_bad", [1]), "address1.wast:289");

// address1.wast:290
assert_trap(() => call($1, "16u_bad", [1]), "address1.wast:290");

// address1.wast:291
assert_trap(() => call($1, "16s_bad", [1]), "address1.wast:291");

// address1.wast:292
assert_trap(() => call($1, "32u_bad", [0]), "address1.wast:292");

// address1.wast:293
assert_trap(() => call($1, "32s_bad", [0]), "address1.wast:293");

// address1.wast:294
assert_trap(() => call($1, "64_bad", [1]), "address1.wast:294");
reinitializeRegistry();
})();
