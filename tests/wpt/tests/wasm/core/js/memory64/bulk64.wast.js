(function bulk64_wast_js() {

// bulk64.wast:2
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x05\x83\x80\x80\x80\x00\x01\x04\x01\x0b\x86\x80\x80\x80\x00\x01\x01\x03\x66\x6f\x6f", "bulk64.wast:2");

// bulk64.wast:2
let $1 = instance($$1);

// bulk64.wast:7
let $$2 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7e\x7f\x7e\x00\x60\x01\x7e\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x83\x80\x80\x80\x00\x01\x04\x01\x07\x92\x80\x80\x80\x00\x02\x04\x66\x69\x6c\x6c\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9d\x80\x80\x80\x00\x02\x8b\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0b\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b", "bulk64.wast:7");

// bulk64.wast:7
let $2 = instance($$2);

// bulk64.wast:21
run(() => call($2, "fill", [1n, 255, 3n]), "bulk64.wast:21");

// bulk64.wast:22
assert_return(() => call($2, "load8_u", [0n]), "bulk64.wast:22", 0);

// bulk64.wast:23
assert_return(() => call($2, "load8_u", [1n]), "bulk64.wast:23", 255);

// bulk64.wast:24
assert_return(() => call($2, "load8_u", [2n]), "bulk64.wast:24", 255);

// bulk64.wast:25
assert_return(() => call($2, "load8_u", [3n]), "bulk64.wast:25", 255);

// bulk64.wast:26
assert_return(() => call($2, "load8_u", [4n]), "bulk64.wast:26", 0);

// bulk64.wast:29
run(() => call($2, "fill", [0n, 48_042, 2n]), "bulk64.wast:29");

// bulk64.wast:30
assert_return(() => call($2, "load8_u", [0n]), "bulk64.wast:30", 170);

// bulk64.wast:31
assert_return(() => call($2, "load8_u", [1n]), "bulk64.wast:31", 170);

// bulk64.wast:34
run(() => call($2, "fill", [0n, 0, 65_536n]), "bulk64.wast:34");

// bulk64.wast:37
run(() => call($2, "fill", [65_536n, 0, 0n]), "bulk64.wast:37");

// bulk64.wast:40
assert_trap(() => call($2, "fill", [65_537n, 0, 0n]), "bulk64.wast:40");

// bulk64.wast:45
let $$3 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7e\x7e\x7e\x00\x60\x01\x7e\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x84\x80\x80\x80\x00\x01\x05\x01\x01\x07\x92\x80\x80\x80\x00\x02\x04\x63\x6f\x70\x79\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x8a\x80\x80\x80\x00\x01\x00\x42\x00\x0b\x04\xaa\xbb\xcc\xdd", "bulk64.wast:45");

// bulk64.wast:45
let $3 = instance($$3);

// bulk64.wast:60
run(() => call($3, "copy", [10n, 0n, 4n]), "bulk64.wast:60");

// bulk64.wast:62
assert_return(() => call($3, "load8_u", [9n]), "bulk64.wast:62", 0);

// bulk64.wast:63
assert_return(() => call($3, "load8_u", [10n]), "bulk64.wast:63", 170);

// bulk64.wast:64
assert_return(() => call($3, "load8_u", [11n]), "bulk64.wast:64", 187);

// bulk64.wast:65
assert_return(() => call($3, "load8_u", [12n]), "bulk64.wast:65", 204);

// bulk64.wast:66
assert_return(() => call($3, "load8_u", [13n]), "bulk64.wast:66", 221);

// bulk64.wast:67
assert_return(() => call($3, "load8_u", [14n]), "bulk64.wast:67", 0);

// bulk64.wast:70
run(() => call($3, "copy", [8n, 10n, 4n]), "bulk64.wast:70");

// bulk64.wast:71
assert_return(() => call($3, "load8_u", [8n]), "bulk64.wast:71", 170);

// bulk64.wast:72
assert_return(() => call($3, "load8_u", [9n]), "bulk64.wast:72", 187);

// bulk64.wast:73
assert_return(() => call($3, "load8_u", [10n]), "bulk64.wast:73", 204);

// bulk64.wast:74
assert_return(() => call($3, "load8_u", [11n]), "bulk64.wast:74", 221);

// bulk64.wast:75
assert_return(() => call($3, "load8_u", [12n]), "bulk64.wast:75", 204);

// bulk64.wast:76
assert_return(() => call($3, "load8_u", [13n]), "bulk64.wast:76", 221);

// bulk64.wast:79
run(() => call($3, "copy", [10n, 7n, 6n]), "bulk64.wast:79");

// bulk64.wast:80
assert_return(() => call($3, "load8_u", [10n]), "bulk64.wast:80", 0);

// bulk64.wast:81
assert_return(() => call($3, "load8_u", [11n]), "bulk64.wast:81", 170);

// bulk64.wast:82
assert_return(() => call($3, "load8_u", [12n]), "bulk64.wast:82", 187);

// bulk64.wast:83
assert_return(() => call($3, "load8_u", [13n]), "bulk64.wast:83", 204);

// bulk64.wast:84
assert_return(() => call($3, "load8_u", [14n]), "bulk64.wast:84", 221);

// bulk64.wast:85
assert_return(() => call($3, "load8_u", [15n]), "bulk64.wast:85", 204);

// bulk64.wast:86
assert_return(() => call($3, "load8_u", [16n]), "bulk64.wast:86", 0);

// bulk64.wast:89
assert_trap(() => call($3, "copy", [13n, 11n, -1n]), "bulk64.wast:89");

// bulk64.wast:92
assert_return(() => call($3, "load8_u", [10n]), "bulk64.wast:92", 0);

// bulk64.wast:93
assert_return(() => call($3, "load8_u", [11n]), "bulk64.wast:93", 170);

// bulk64.wast:94
assert_return(() => call($3, "load8_u", [12n]), "bulk64.wast:94", 187);

// bulk64.wast:95
assert_return(() => call($3, "load8_u", [13n]), "bulk64.wast:95", 204);

// bulk64.wast:96
assert_return(() => call($3, "load8_u", [14n]), "bulk64.wast:96", 221);

// bulk64.wast:97
assert_return(() => call($3, "load8_u", [15n]), "bulk64.wast:97", 204);

// bulk64.wast:98
assert_return(() => call($3, "load8_u", [16n]), "bulk64.wast:98", 0);

// bulk64.wast:101
run(() => call($3, "copy", [65_280n, 0n, 256n]), "bulk64.wast:101");

// bulk64.wast:102
run(() => call($3, "copy", [65_024n, 65_280n, 256n]), "bulk64.wast:102");

// bulk64.wast:105
run(() => call($3, "copy", [65_536n, 0n, 0n]), "bulk64.wast:105");

// bulk64.wast:106
run(() => call($3, "copy", [0n, 65_536n, 0n]), "bulk64.wast:106");

// bulk64.wast:109
assert_trap(() => call($3, "copy", [65_537n, 0n, 0n]), "bulk64.wast:109");

// bulk64.wast:112
assert_trap(() => call($3, "copy", [0n, 65_537n, 0n]), "bulk64.wast:112");

// bulk64.wast:117
let $$4 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7e\x7f\x7f\x00\x60\x01\x7e\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x83\x80\x80\x80\x00\x01\x04\x01\x07\x92\x80\x80\x80\x00\x02\x04\x69\x6e\x69\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0c\x81\x80\x80\x80\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x08\x00\x00\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\x87\x80\x80\x80\x00\x01\x01\x04\xaa\xbb\xcc\xdd", "bulk64.wast:117");

// bulk64.wast:117
let $4 = instance($$4);

// bulk64.wast:131
run(() => call($4, "init", [0n, 1, 2]), "bulk64.wast:131");

// bulk64.wast:132
assert_return(() => call($4, "load8_u", [0n]), "bulk64.wast:132", 187);

// bulk64.wast:133
assert_return(() => call($4, "load8_u", [1n]), "bulk64.wast:133", 204);

// bulk64.wast:134
assert_return(() => call($4, "load8_u", [2n]), "bulk64.wast:134", 0);

// bulk64.wast:137
run(() => call($4, "init", [65_532n, 0, 4]), "bulk64.wast:137");

// bulk64.wast:140
assert_trap(() => call($4, "init", [65_534n, 0, 3]), "bulk64.wast:140");

// bulk64.wast:142
assert_return(() => call($4, "load8_u", [65_534n]), "bulk64.wast:142", 204);

// bulk64.wast:143
assert_return(() => call($4, "load8_u", [65_535n]), "bulk64.wast:143", 221);

// bulk64.wast:146
run(() => call($4, "init", [65_536n, 0, 0]), "bulk64.wast:146");

// bulk64.wast:147
run(() => call($4, "init", [0n, 4, 0]), "bulk64.wast:147");

// bulk64.wast:150
assert_trap(() => call($4, "init", [65_537n, 0, 0]), "bulk64.wast:150");

// bulk64.wast:153
assert_trap(() => call($4, "init", [0n, 5, 0]), "bulk64.wast:153");

// bulk64.wast:158
run(() => call($4, "init", [0n, 0, 0]), "bulk64.wast:158");

// bulk64.wast:161
let $$5 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x85\x80\x80\x80\x00\x04\x00\x00\x00\x00\x05\x83\x80\x80\x80\x00\x01\x04\x01\x07\xbb\x80\x80\x80\x00\x04\x0c\x64\x72\x6f\x70\x5f\x70\x61\x73\x73\x69\x76\x65\x00\x00\x0c\x69\x6e\x69\x74\x5f\x70\x61\x73\x73\x69\x76\x65\x00\x01\x0b\x64\x72\x6f\x70\x5f\x61\x63\x74\x69\x76\x65\x00\x02\x0b\x69\x6e\x69\x74\x5f\x61\x63\x74\x69\x76\x65\x00\x03\x0c\x81\x80\x80\x80\x00\x02\x0a\xb7\x80\x80\x80\x00\x04\x85\x80\x80\x80\x00\x00\xfc\x09\x00\x0b\x8c\x80\x80\x80\x00\x00\x42\x00\x41\x00\x41\x00\xfc\x08\x00\x00\x0b\x85\x80\x80\x80\x00\x00\xfc\x09\x01\x0b\x8c\x80\x80\x80\x00\x00\x42\x00\x41\x00\x41\x00\xfc\x08\x01\x00\x0b\x0b\x88\x80\x80\x80\x00\x02\x01\x00\x00\x42\x00\x0b\x00", "bulk64.wast:161");

// bulk64.wast:161
let $5 = instance($$5);

// bulk64.wast:176
run(() => call($5, "init_passive", []), "bulk64.wast:176");

// bulk64.wast:177
run(() => call($5, "drop_passive", []), "bulk64.wast:177");

// bulk64.wast:178
run(() => call($5, "drop_passive", []), "bulk64.wast:178");

// bulk64.wast:179
run(() => call($5, "drop_active", []), "bulk64.wast:179");
reinitializeRegistry();
})();
