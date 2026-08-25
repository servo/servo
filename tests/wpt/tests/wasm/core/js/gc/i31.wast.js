(function i31_wast_js() {

// i31.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x99\x80\x80\x80\x00\x05\x60\x01\x7f\x01\x64\x6c\x60\x01\x7f\x01\x7f\x60\x00\x01\x7f\x60\x00\x02\x7f\x7f\x60\x01\x7f\x00\x03\x88\x80\x80\x80\x00\x07\x00\x01\x01\x02\x02\x03\x04\x06\x91\x80\x80\x80\x00\x02\x64\x6c\x00\x41\x02\xfb\x1c\x0b\x64\x6c\x01\x41\x03\xfb\x1c\x0b\x07\xcc\x80\x80\x80\x00\x07\x03\x6e\x65\x77\x00\x00\x05\x67\x65\x74\x5f\x75\x00\x01\x05\x67\x65\x74\x5f\x73\x00\x02\x0a\x67\x65\x74\x5f\x75\x2d\x6e\x75\x6c\x6c\x00\x03\x0a\x67\x65\x74\x5f\x73\x2d\x6e\x75\x6c\x6c\x00\x04\x0b\x67\x65\x74\x5f\x67\x6c\x6f\x62\x61\x6c\x73\x00\x05\x0a\x73\x65\x74\x5f\x67\x6c\x6f\x62\x61\x6c\x00\x06\x0a\xd8\x80\x80\x80\x00\x07\x86\x80\x80\x80\x00\x00\x20\x00\xfb\x1c\x0b\x88\x80\x80\x80\x00\x00\x20\x00\xfb\x1c\xfb\x1e\x0b\x88\x80\x80\x80\x00\x00\x20\x00\xfb\x1c\xfb\x1d\x0b\x86\x80\x80\x80\x00\x00\xd0\x6c\xfb\x1e\x0b\x86\x80\x80\x80\x00\x00\xd0\x6c\xfb\x1d\x0b\x8a\x80\x80\x80\x00\x00\x23\x00\xfb\x1e\x23\x01\xfb\x1e\x0b\x88\x80\x80\x80\x00\x00\x20\x00\xfb\x1c\x24\x01\x0b", "i31.wast:1");

// i31.wast:1
let $1 = instance($$1);

// i31.wast:33
assert_return(() => call($1, "new", [1]), "i31.wast:33", "ref.i31");

// i31.wast:35
assert_return(() => call($1, "get_u", [0]), "i31.wast:35", 0);

// i31.wast:36
assert_return(() => call($1, "get_u", [100]), "i31.wast:36", 100);

// i31.wast:37
assert_return(() => call($1, "get_u", [-1]), "i31.wast:37", 2_147_483_647);

// i31.wast:38
assert_return(() => call($1, "get_u", [1_073_741_823]), "i31.wast:38", 1_073_741_823);

// i31.wast:39
assert_return(() => call($1, "get_u", [1_073_741_824]), "i31.wast:39", 1_073_741_824);

// i31.wast:40
assert_return(() => call($1, "get_u", [2_147_483_647]), "i31.wast:40", 2_147_483_647);

// i31.wast:41
assert_return(() => call($1, "get_u", [-1_431_655_766]), "i31.wast:41", 715_827_882);

// i31.wast:42
assert_return(() => call($1, "get_u", [-894_784_854]), "i31.wast:42", 1_252_698_794);

// i31.wast:44
assert_return(() => call($1, "get_s", [0]), "i31.wast:44", 0);

// i31.wast:45
assert_return(() => call($1, "get_s", [100]), "i31.wast:45", 100);

// i31.wast:46
assert_return(() => call($1, "get_s", [-1]), "i31.wast:46", -1);

// i31.wast:47
assert_return(() => call($1, "get_s", [1_073_741_823]), "i31.wast:47", 1_073_741_823);

// i31.wast:48
assert_return(() => call($1, "get_s", [1_073_741_824]), "i31.wast:48", -1_073_741_824);

// i31.wast:49
assert_return(() => call($1, "get_s", [2_147_483_647]), "i31.wast:49", -1);

// i31.wast:50
assert_return(() => call($1, "get_s", [-1_431_655_766]), "i31.wast:50", 715_827_882);

// i31.wast:51
assert_return(() => call($1, "get_s", [-894_784_854]), "i31.wast:51", -894_784_854);

// i31.wast:53
assert_trap(() => call($1, "get_u-null", []), "i31.wast:53");

// i31.wast:54
assert_trap(() => call($1, "get_s-null", []), "i31.wast:54");

// i31.wast:56
assert_return(() => call($1, "get_globals", []), "i31.wast:56", 2, 3);

// i31.wast:58
run(() => call($1, "set_global", [1_234]), "i31.wast:58");

// i31.wast:59
assert_return(() => call($1, "get_globals", []), "i31.wast:59", 2, 1_234);

// i31.wast:61
let $$2 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x96\x80\x80\x80\x00\x04\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x02\x7f\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x87\x80\x80\x80\x00\x06\x00\x01\x02\x03\x03\x03\x04\x85\x80\x80\x80\x00\x01\x6c\x01\x03\x0a\x07\xaa\x80\x80\x80\x00\x06\x04\x73\x69\x7a\x65\x00\x00\x03\x67\x65\x74\x00\x01\x04\x67\x72\x6f\x77\x00\x02\x04\x66\x69\x6c\x6c\x00\x03\x04\x63\x6f\x70\x79\x00\x04\x04\x69\x6e\x69\x74\x00\x05\x09\xaf\x80\x80\x80\x00\x02\x06\x00\x41\x00\x0b\x6c\x03\x41\xe7\x07\xfb\x1c\x0b\x41\xf8\x06\xfb\x1c\x0b\x41\x89\x06\xfb\x1c\x0b\x05\x6c\x03\x41\xfb\x00\xfb\x1c\x0b\x41\xc8\x03\xfb\x1c\x0b\x41\x95\x06\xfb\x1c\x0b\x0a\xdc\x80\x80\x80\x00\x06\x85\x80\x80\x80\x00\x00\xfc\x10\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x25\x00\xfb\x1e\x0b\x8b\x80\x80\x80\x00\x00\x20\x01\xfb\x1c\x20\x00\xfc\x0f\x00\x0b\x8d\x80\x80\x80\x00\x00\x20\x00\x20\x01\xfb\x1c\x20\x02\xfc\x11\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0c\x01\x00\x0b", "i31.wast:61");
let $tables_of_i31ref = $$2;

// i31.wast:61
let $2 = instance($tables_of_i31ref);
let tables_of_i31ref = $2;

// i31.wast:96
assert_return(() => call($2, "size", []), "i31.wast:96", 3);

// i31.wast:97
assert_return(() => call($2, "get", [0]), "i31.wast:97", 999);

// i31.wast:98
assert_return(() => call($2, "get", [1]), "i31.wast:98", 888);

// i31.wast:99
assert_return(() => call($2, "get", [2]), "i31.wast:99", 777);

// i31.wast:102
assert_return(() => call($2, "grow", [2, 333]), "i31.wast:102", 3);

// i31.wast:103
assert_return(() => call($2, "size", []), "i31.wast:103", 5);

// i31.wast:104
assert_return(() => call($2, "get", [3]), "i31.wast:104", 333);

// i31.wast:105
assert_return(() => call($2, "get", [4]), "i31.wast:105", 333);

// i31.wast:108
run(() => call($2, "fill", [2, 111, 2]), "i31.wast:108");

// i31.wast:109
assert_return(() => call($2, "get", [2]), "i31.wast:109", 111);

// i31.wast:110
assert_return(() => call($2, "get", [3]), "i31.wast:110", 111);

// i31.wast:113
run(() => call($2, "copy", [3, 0, 2]), "i31.wast:113");

// i31.wast:114
assert_return(() => call($2, "get", [3]), "i31.wast:114", 999);

// i31.wast:115
assert_return(() => call($2, "get", [4]), "i31.wast:115", 888);

// i31.wast:118
run(() => call($2, "init", [1, 0, 3]), "i31.wast:118");

// i31.wast:119
assert_return(() => call($2, "get", [1]), "i31.wast:119", 123);

// i31.wast:120
assert_return(() => call($2, "get", [2]), "i31.wast:120", 456);

// i31.wast:121
assert_return(() => call($2, "get", [3]), "i31.wast:121", 789);

// i31.wast:123
let $$3 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x06\x86\x80\x80\x80\x00\x01\x7f\x00\x41\x2a\x0b\x07\x85\x80\x80\x80\x00\x01\x01\x67\x03\x00", "i31.wast:123");
let $env = $$3;

// i31.wast:123
let $3 = instance($env);
let env = $3;

// i31.wast:126
register("env", $3)

// i31.wast:128
let $$4 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x86\x80\x80\x80\x00\x01\x60\x01\x7f\x01\x7f\x02\x8a\x80\x80\x80\x00\x01\x03\x65\x6e\x76\x01\x67\x03\x7f\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x8d\x80\x80\x80\x00\x01\x40\x00\x64\x6c\x01\x03\x03\x23\x00\xfb\x1c\x0b\x07\x87\x80\x80\x80\x00\x01\x03\x67\x65\x74\x00\x00\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x00\x20\x00\x25\x00\xfb\x1e\x0b", "i31.wast:128");
let $i31ref_of_global_table_initializer = $$4;

// i31.wast:128
let $4 = instance($i31ref_of_global_table_initializer);
let i31ref_of_global_table_initializer = $4;

// i31.wast:136
assert_return(() => call($4, "get", [0]), "i31.wast:136", 42);

// i31.wast:137
assert_return(() => call($4, "get", [1]), "i31.wast:137", 42);

// i31.wast:138
assert_return(() => call($4, "get", [2]), "i31.wast:138", 42);

// i31.wast:140
let $$5 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x02\x8a\x80\x80\x80\x00\x01\x03\x65\x6e\x76\x01\x67\x03\x7f\x00\x03\x82\x80\x80\x80\x00\x01\x00\x06\x88\x80\x80\x80\x00\x01\x6c\x00\x23\x00\xfb\x1c\x0b\x07\x87\x80\x80\x80\x00\x01\x03\x67\x65\x74\x00\x00\x0a\x8c\x80\x80\x80\x00\x01\x86\x80\x80\x80\x00\x00\x23\x01\xfb\x1e\x0b", "i31.wast:140");
let $i31ref_of_global_global_initializer = $$5;

// i31.wast:140
let $5 = instance($i31ref_of_global_global_initializer);
let i31ref_of_global_global_initializer = $5;

// i31.wast:148
assert_return(() => call($5, "get", []), "i31.wast:148", 42);

// i31.wast:150
let $$6 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8a\x80\x80\x80\x00\x02\x60\x00\x02\x7f\x7f\x60\x01\x7f\x00\x03\x83\x80\x80\x80\x00\x02\x00\x01\x06\x91\x80\x80\x80\x00\x02\x6e\x00\x41\xd2\x09\xfb\x1c\x0b\x6e\x01\x41\xae\x2c\xfb\x1c\x0b\x07\x9c\x80\x80\x80\x00\x02\x0b\x67\x65\x74\x5f\x67\x6c\x6f\x62\x61\x6c\x73\x00\x00\x0a\x73\x65\x74\x5f\x67\x6c\x6f\x62\x61\x6c\x00\x01\x0a\xa3\x80\x80\x80\x00\x02\x90\x80\x80\x80\x00\x00\x23\x00\xfb\x17\x6c\xfb\x1e\x23\x01\xfb\x17\x6c\xfb\x1e\x0b\x88\x80\x80\x80\x00\x00\x20\x00\xfb\x1c\x24\x01\x0b", "i31.wast:150");
let $anyref_global_of_i31ref = $$6;

// i31.wast:150
let $6 = instance($anyref_global_of_i31ref);
let anyref_global_of_i31ref = $6;

// i31.wast:164
assert_return(() => call($6, "get_globals", []), "i31.wast:164", 1_234, 5_678);

// i31.wast:165
run(() => call($6, "set_global", [0]), "i31.wast:165");

// i31.wast:166
assert_return(() => call($6, "get_globals", []), "i31.wast:166", 1_234, 0);

// i31.wast:168
let $$7 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x96\x80\x80\x80\x00\x04\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x60\x02\x7f\x7f\x01\x7f\x60\x03\x7f\x7f\x7f\x00\x03\x87\x80\x80\x80\x00\x06\x00\x01\x02\x03\x03\x03\x04\x85\x80\x80\x80\x00\x01\x6e\x01\x03\x0a\x07\xaa\x80\x80\x80\x00\x06\x04\x73\x69\x7a\x65\x00\x00\x03\x67\x65\x74\x00\x01\x04\x67\x72\x6f\x77\x00\x02\x04\x66\x69\x6c\x6c\x00\x03\x04\x63\x6f\x70\x79\x00\x04\x04\x69\x6e\x69\x74\x00\x05\x09\xaf\x80\x80\x80\x00\x02\x06\x00\x41\x00\x0b\x6c\x03\x41\xe7\x07\xfb\x1c\x0b\x41\xf8\x06\xfb\x1c\x0b\x41\x89\x06\xfb\x1c\x0b\x05\x6c\x03\x41\xfb\x00\xfb\x1c\x0b\x41\xc8\x03\xfb\x1c\x0b\x41\x95\x06\xfb\x1c\x0b\x0a\xdf\x80\x80\x80\x00\x06\x85\x80\x80\x80\x00\x00\xfc\x10\x00\x0b\x8b\x80\x80\x80\x00\x00\x20\x00\x25\x00\xfb\x17\x6c\xfb\x1e\x0b\x8b\x80\x80\x80\x00\x00\x20\x01\xfb\x1c\x20\x00\xfc\x0f\x00\x0b\x8d\x80\x80\x80\x00\x00\x20\x00\x20\x01\xfb\x1c\x20\x02\xfc\x11\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0e\x00\x00\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0c\x01\x00\x0b", "i31.wast:168");
let $anyref_table_of_i31ref = $$7;

// i31.wast:168
let $7 = instance($anyref_table_of_i31ref);
let anyref_table_of_i31ref = $7;

// i31.wast:203
assert_return(() => call($7, "size", []), "i31.wast:203", 3);

// i31.wast:204
assert_return(() => call($7, "get", [0]), "i31.wast:204", 999);

// i31.wast:205
assert_return(() => call($7, "get", [1]), "i31.wast:205", 888);

// i31.wast:206
assert_return(() => call($7, "get", [2]), "i31.wast:206", 777);

// i31.wast:209
assert_return(() => call($7, "grow", [2, 333]), "i31.wast:209", 3);

// i31.wast:210
assert_return(() => call($7, "size", []), "i31.wast:210", 5);

// i31.wast:211
assert_return(() => call($7, "get", [3]), "i31.wast:211", 333);

// i31.wast:212
assert_return(() => call($7, "get", [4]), "i31.wast:212", 333);

// i31.wast:215
run(() => call($7, "fill", [2, 111, 2]), "i31.wast:215");

// i31.wast:216
assert_return(() => call($7, "get", [2]), "i31.wast:216", 111);

// i31.wast:217
assert_return(() => call($7, "get", [3]), "i31.wast:217", 111);

// i31.wast:220
run(() => call($7, "copy", [3, 0, 2]), "i31.wast:220");

// i31.wast:221
assert_return(() => call($7, "get", [3]), "i31.wast:221", 999);

// i31.wast:222
assert_return(() => call($7, "get", [4]), "i31.wast:222", 888);

// i31.wast:225
run(() => call($7, "init", [1, 0, 3]), "i31.wast:225");

// i31.wast:226
assert_return(() => call($7, "get", [1]), "i31.wast:226", 123);

// i31.wast:227
assert_return(() => call($7, "get", [2]), "i31.wast:227", 456);

// i31.wast:228
assert_return(() => call($7, "get", [3]), "i31.wast:228", 789);
reinitializeRegistry();
})();
