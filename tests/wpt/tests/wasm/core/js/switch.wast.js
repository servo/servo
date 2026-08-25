(function switch_wast_js() {

// switch.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8f\x80\x80\x80\x00\x03\x60\x01\x7f\x01\x7f\x60\x01\x7e\x01\x7e\x60\x00\x01\x7f\x03\x85\x80\x80\x80\x00\x04\x00\x01\x00\x02\x07\x9e\x80\x80\x80\x00\x04\x04\x73\x74\x6d\x74\x00\x00\x04\x65\x78\x70\x72\x00\x01\x03\x61\x72\x67\x00\x02\x06\x63\x6f\x72\x6e\x65\x72\x00\x03\x0a\xee\x81\x80\x80\x00\x04\xd7\x80\x80\x80\x00\x01\x01\x7f\x41\xe4\x00\x21\x01\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x20\x00\x0e\x08\x00\x01\x02\x03\x04\x05\x06\x08\x07\x0b\x20\x00\x0f\x0b\x01\x0b\x0b\x41\x00\x20\x00\x6b\x21\x01\x0c\x05\x0b\x0c\x04\x0b\x41\xe5\x00\x21\x01\x0c\x03\x0b\x41\xe5\x00\x21\x01\x0b\x41\xe6\x00\x21\x01\x0b\x0b\x20\x01\x0f\x0b\xcc\x80\x80\x80\x00\x01\x01\x7e\x42\xe4\x00\x21\x01\x02\x7e\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x02\x40\x20\x00\xa7\x0e\x08\x00\x01\x02\x03\x06\x05\x04\x08\x07\x0b\x20\x00\x0f\x0b\x01\x0b\x0b\x42\x00\x20\x00\x7d\x0c\x05\x0b\x42\xe5\x00\x21\x01\x0b\x0b\x0b\x20\x01\x0c\x01\x0b\x42\x7b\x0b\x0f\x0b\xaa\x80\x80\x80\x00\x00\x02\x7f\x41\x0a\x02\x7f\x41\xe4\x00\x02\x7f\x41\xe8\x07\x02\x7f\x41\x02\x20\x00\x6c\x41\x03\x20\x00\x71\x0e\x03\x01\x02\x03\x00\x0b\x6a\x0b\x6a\x0b\x6a\x0b\x0f\x0b\x8c\x80\x80\x80\x00\x00\x02\x40\x41\x00\x0e\x00\x00\x0b\x41\x01\x0b", "switch.wast:1");

// switch.wast:1
let $1 = instance($$1);

// switch.wast:120
assert_return(() => call($1, "stmt", [0]), "switch.wast:120", 0);

// switch.wast:121
assert_return(() => call($1, "stmt", [1]), "switch.wast:121", -1);

// switch.wast:122
assert_return(() => call($1, "stmt", [2]), "switch.wast:122", -2);

// switch.wast:123
assert_return(() => call($1, "stmt", [3]), "switch.wast:123", -3);

// switch.wast:124
assert_return(() => call($1, "stmt", [4]), "switch.wast:124", 100);

// switch.wast:125
assert_return(() => call($1, "stmt", [5]), "switch.wast:125", 101);

// switch.wast:126
assert_return(() => call($1, "stmt", [6]), "switch.wast:126", 102);

// switch.wast:127
assert_return(() => call($1, "stmt", [7]), "switch.wast:127", 100);

// switch.wast:128
assert_return(() => call($1, "stmt", [-10]), "switch.wast:128", 102);

// switch.wast:130
assert_return(() => call($1, "expr", [0n]), "switch.wast:130", 0n);

// switch.wast:131
assert_return(() => call($1, "expr", [1n]), "switch.wast:131", -1n);

// switch.wast:132
assert_return(() => call($1, "expr", [2n]), "switch.wast:132", -2n);

// switch.wast:133
assert_return(() => call($1, "expr", [3n]), "switch.wast:133", -3n);

// switch.wast:134
assert_return(() => call($1, "expr", [6n]), "switch.wast:134", 101n);

// switch.wast:135
assert_return(() => call($1, "expr", [7n]), "switch.wast:135", -5n);

// switch.wast:136
assert_return(() => call($1, "expr", [-10n]), "switch.wast:136", 100n);

// switch.wast:138
assert_return(() => call($1, "arg", [0]), "switch.wast:138", 110);

// switch.wast:139
assert_return(() => call($1, "arg", [1]), "switch.wast:139", 12);

// switch.wast:140
assert_return(() => call($1, "arg", [2]), "switch.wast:140", 4);

// switch.wast:141
assert_return(() => call($1, "arg", [3]), "switch.wast:141", 1_116);

// switch.wast:142
assert_return(() => call($1, "arg", [4]), "switch.wast:142", 118);

// switch.wast:143
assert_return(() => call($1, "arg", [5]), "switch.wast:143", 20);

// switch.wast:144
assert_return(() => call($1, "arg", [6]), "switch.wast:144", 12);

// switch.wast:145
assert_return(() => call($1, "arg", [7]), "switch.wast:145", 1_124);

// switch.wast:146
assert_return(() => call($1, "arg", [8]), "switch.wast:146", 126);

// switch.wast:148
assert_return(() => call($1, "corner", []), "switch.wast:148", 1);

// switch.wast:150
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8d\x80\x80\x80\x00\x01\x87\x80\x80\x80\x00\x00\x41\x00\x0e\x00\x03\x0b", "switch.wast:150");
reinitializeRegistry();
})();
