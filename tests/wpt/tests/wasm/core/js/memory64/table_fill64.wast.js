(function table_fill64_wast_js() {

// table_fill64.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x97\x80\x80\x80\x00\x04\x60\x03\x7f\x6f\x7f\x00\x60\x01\x7f\x01\x6f\x60\x03\x7e\x6f\x7e\x00\x60\x01\x7e\x01\x6f\x03\x86\x80\x80\x80\x00\x05\x00\x00\x01\x02\x03\x04\x87\x80\x80\x80\x00\x02\x6f\x00\x0a\x6f\x04\x0a\x07\xb1\x80\x80\x80\x00\x05\x04\x66\x69\x6c\x6c\x00\x00\x0b\x66\x69\x6c\x6c\x2d\x61\x62\x62\x72\x65\x76\x00\x01\x03\x67\x65\x74\x00\x02\x08\x66\x69\x6c\x6c\x2d\x74\x36\x34\x00\x03\x07\x67\x65\x74\x2d\x74\x36\x34\x00\x04\x0a\xc7\x80\x80\x80\x00\x05\x8b\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x11\x00\x0b\x8b\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x11\x00\x0b\x86\x80\x80\x80\x00\x00\x20\x00\x25\x00\x0b\x8b\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x11\x01\x0b\x86\x80\x80\x80\x00\x00\x20\x00\x25\x01\x0b", "table_fill64.wast:1");

// table_fill64.wast:1
let $1 = instance($$1);

// table_fill64.wast:27
assert_return(() => call($1, "get", [1]), "table_fill64.wast:27", null);

// table_fill64.wast:28
assert_return(() => call($1, "get", [2]), "table_fill64.wast:28", null);

// table_fill64.wast:29
assert_return(() => call($1, "get", [3]), "table_fill64.wast:29", null);

// table_fill64.wast:30
assert_return(() => call($1, "get", [4]), "table_fill64.wast:30", null);

// table_fill64.wast:31
assert_return(() => call($1, "get", [5]), "table_fill64.wast:31", null);

// table_fill64.wast:33
assert_return(() => call($1, "fill", [2, hostref(1), 3]), "table_fill64.wast:33");

// table_fill64.wast:34
assert_return(() => call($1, "get", [1]), "table_fill64.wast:34", null);

// table_fill64.wast:35
assert_return(() => call($1, "get", [2]), "table_fill64.wast:35", hostref(1));

// table_fill64.wast:36
assert_return(() => call($1, "get", [3]), "table_fill64.wast:36", hostref(1));

// table_fill64.wast:37
assert_return(() => call($1, "get", [4]), "table_fill64.wast:37", hostref(1));

// table_fill64.wast:38
assert_return(() => call($1, "get", [5]), "table_fill64.wast:38", null);

// table_fill64.wast:40
assert_return(() => call($1, "fill", [4, hostref(2), 2]), "table_fill64.wast:40");

// table_fill64.wast:41
assert_return(() => call($1, "get", [3]), "table_fill64.wast:41", hostref(1));

// table_fill64.wast:42
assert_return(() => call($1, "get", [4]), "table_fill64.wast:42", hostref(2));

// table_fill64.wast:43
assert_return(() => call($1, "get", [5]), "table_fill64.wast:43", hostref(2));

// table_fill64.wast:44
assert_return(() => call($1, "get", [6]), "table_fill64.wast:44", null);

// table_fill64.wast:46
assert_return(() => call($1, "fill", [4, hostref(3), 0]), "table_fill64.wast:46");

// table_fill64.wast:47
assert_return(() => call($1, "get", [3]), "table_fill64.wast:47", hostref(1));

// table_fill64.wast:48
assert_return(() => call($1, "get", [4]), "table_fill64.wast:48", hostref(2));

// table_fill64.wast:49
assert_return(() => call($1, "get", [5]), "table_fill64.wast:49", hostref(2));

// table_fill64.wast:51
assert_return(() => call($1, "fill", [8, hostref(4), 2]), "table_fill64.wast:51");

// table_fill64.wast:52
assert_return(() => call($1, "get", [7]), "table_fill64.wast:52", null);

// table_fill64.wast:53
assert_return(() => call($1, "get", [8]), "table_fill64.wast:53", hostref(4));

// table_fill64.wast:54
assert_return(() => call($1, "get", [9]), "table_fill64.wast:54", hostref(4));

// table_fill64.wast:56
assert_return(() => call($1, "fill-abbrev", [9, null, 1]), "table_fill64.wast:56");

// table_fill64.wast:57
assert_return(() => call($1, "get", [8]), "table_fill64.wast:57", hostref(4));

// table_fill64.wast:58
assert_return(() => call($1, "get", [9]), "table_fill64.wast:58", null);

// table_fill64.wast:60
assert_return(() => call($1, "fill", [10, hostref(5), 0]), "table_fill64.wast:60");

// table_fill64.wast:61
assert_return(() => call($1, "get", [9]), "table_fill64.wast:61", null);

// table_fill64.wast:63
assert_trap(() => call($1, "fill", [8, hostref(6), 3]), "table_fill64.wast:63");

// table_fill64.wast:67
assert_return(() => call($1, "get", [7]), "table_fill64.wast:67", null);

// table_fill64.wast:68
assert_return(() => call($1, "get", [8]), "table_fill64.wast:68", hostref(4));

// table_fill64.wast:69
assert_return(() => call($1, "get", [9]), "table_fill64.wast:69", null);

// table_fill64.wast:71
assert_trap(() => call($1, "fill", [11, null, 0]), "table_fill64.wast:71");

// table_fill64.wast:76
assert_trap(() => call($1, "fill", [11, null, 10]), "table_fill64.wast:76");

// table_fill64.wast:83
assert_return(() => call($1, "get-t64", [1n]), "table_fill64.wast:83", null);

// table_fill64.wast:84
assert_return(() => call($1, "get-t64", [2n]), "table_fill64.wast:84", null);

// table_fill64.wast:85
assert_return(() => call($1, "get-t64", [3n]), "table_fill64.wast:85", null);

// table_fill64.wast:86
assert_return(() => call($1, "get-t64", [4n]), "table_fill64.wast:86", null);

// table_fill64.wast:87
assert_return(() => call($1, "get-t64", [5n]), "table_fill64.wast:87", null);

// table_fill64.wast:89
assert_return(() => call($1, "fill-t64", [2n, hostref(1), 3n]), "table_fill64.wast:89");

// table_fill64.wast:90
assert_return(() => call($1, "get-t64", [1n]), "table_fill64.wast:90", null);

// table_fill64.wast:91
assert_return(() => call($1, "get-t64", [2n]), "table_fill64.wast:91", hostref(1));

// table_fill64.wast:92
assert_return(() => call($1, "get-t64", [3n]), "table_fill64.wast:92", hostref(1));

// table_fill64.wast:93
assert_return(() => call($1, "get-t64", [4n]), "table_fill64.wast:93", hostref(1));

// table_fill64.wast:94
assert_return(() => call($1, "get-t64", [5n]), "table_fill64.wast:94", null);

// table_fill64.wast:96
assert_return(() => call($1, "fill-t64", [4n, hostref(2), 2n]), "table_fill64.wast:96");

// table_fill64.wast:97
assert_return(() => call($1, "get-t64", [3n]), "table_fill64.wast:97", hostref(1));

// table_fill64.wast:98
assert_return(() => call($1, "get-t64", [4n]), "table_fill64.wast:98", hostref(2));

// table_fill64.wast:99
assert_return(() => call($1, "get-t64", [5n]), "table_fill64.wast:99", hostref(2));

// table_fill64.wast:100
assert_return(() => call($1, "get-t64", [6n]), "table_fill64.wast:100", null);

// table_fill64.wast:102
assert_return(() => call($1, "fill-t64", [4n, hostref(3), 0n]), "table_fill64.wast:102");

// table_fill64.wast:103
assert_return(() => call($1, "get-t64", [3n]), "table_fill64.wast:103", hostref(1));

// table_fill64.wast:104
assert_return(() => call($1, "get-t64", [4n]), "table_fill64.wast:104", hostref(2));

// table_fill64.wast:105
assert_return(() => call($1, "get-t64", [5n]), "table_fill64.wast:105", hostref(2));

// table_fill64.wast:107
assert_return(() => call($1, "fill-t64", [8n, hostref(4), 2n]), "table_fill64.wast:107");

// table_fill64.wast:108
assert_return(() => call($1, "get-t64", [7n]), "table_fill64.wast:108", null);

// table_fill64.wast:109
assert_return(() => call($1, "get-t64", [8n]), "table_fill64.wast:109", hostref(4));

// table_fill64.wast:110
assert_return(() => call($1, "get-t64", [9n]), "table_fill64.wast:110", hostref(4));

// table_fill64.wast:112
assert_return(() => call($1, "fill-t64", [9n, null, 1n]), "table_fill64.wast:112");

// table_fill64.wast:113
assert_return(() => call($1, "get-t64", [8n]), "table_fill64.wast:113", hostref(4));

// table_fill64.wast:114
assert_return(() => call($1, "get-t64", [9n]), "table_fill64.wast:114", null);

// table_fill64.wast:116
assert_return(() => call($1, "fill-t64", [10n, hostref(5), 0n]), "table_fill64.wast:116");

// table_fill64.wast:117
assert_return(() => call($1, "get-t64", [9n]), "table_fill64.wast:117", null);

// table_fill64.wast:119
assert_trap(() => call($1, "fill-t64", [8n, hostref(6), 3n]), "table_fill64.wast:119");

// table_fill64.wast:123
assert_return(() => call($1, "get-t64", [7n]), "table_fill64.wast:123", null);

// table_fill64.wast:124
assert_return(() => call($1, "get-t64", [8n]), "table_fill64.wast:124", hostref(4));

// table_fill64.wast:125
assert_return(() => call($1, "get-t64", [9n]), "table_fill64.wast:125", null);

// table_fill64.wast:127
assert_trap(() => call($1, "fill-t64", [11n, null, 0n]), "table_fill64.wast:127");

// table_fill64.wast:132
assert_trap(() => call($1, "fill-t64", [11n, null, 10n]), "table_fill64.wast:132");

// table_fill64.wast:139
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x6f\x00\x0a\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\xfc\x11\x00\x0b", "table_fill64.wast:139");

// table_fill64.wast:148
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x6f\x00\x0a\x0a\x8f\x80\x80\x80\x00\x01\x89\x80\x80\x80\x00\x00\xd0\x6f\x41\x01\xfc\x11\x00\x0b", "table_fill64.wast:148");

// table_fill64.wast:157
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x6f\x00\x0a\x0a\x8f\x80\x80\x80\x00\x01\x89\x80\x80\x80\x00\x00\x41\x01\x41\x01\xfc\x11\x00\x0b", "table_fill64.wast:157");

// table_fill64.wast:166
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x6f\x00\x0a\x0a\x8f\x80\x80\x80\x00\x01\x89\x80\x80\x80\x00\x00\x41\x01\xd0\x6f\xfc\x11\x00\x0b", "table_fill64.wast:166");

// table_fill64.wast:175
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x6f\x00\x00\x0a\x94\x80\x80\x80\x00\x01\x8e\x80\x80\x80\x00\x00\x43\x00\x00\x80\x3f\xd0\x6f\x41\x01\xfc\x11\x00\x0b", "table_fill64.wast:175");

// table_fill64.wast:184
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x01\x6f\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x70\x00\x00\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x01\x20\x00\x41\x01\xfc\x11\x00\x0b", "table_fill64.wast:184");

// table_fill64.wast:193
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x6f\x00\x00\x0a\x94\x80\x80\x80\x00\x01\x8e\x80\x80\x80\x00\x00\x41\x01\xd0\x6f\x43\x00\x00\x80\x3f\xfc\x11\x00\x0b", "table_fill64.wast:193");

// table_fill64.wast:203
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x01\x6f\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x87\x80\x80\x80\x00\x02\x6f\x00\x01\x70\x00\x01\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x00\x20\x00\x41\x01\xfc\x11\x01\x0b", "table_fill64.wast:203");

// table_fill64.wast:214
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x6f\x00\x01\x0a\x91\x80\x80\x80\x00\x01\x8b\x80\x80\x80\x00\x00\x41\x00\xd0\x6f\x41\x01\xfc\x11\x00\x0b", "table_fill64.wast:214");
reinitializeRegistry();
})();
