(function store2_wast_js() {

// store2.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x05\x83\x80\x80\x80\x00\x01\x00\x02\x07\x87\x80\x80\x80\x00\x01\x03\x6d\x65\x6d\x02\x00", "store2.wast:1");

// store2.wast:1
let $1 = instance($$1);

// store2.wast:4
register("M", $1)

// store2.wast:6
let $$2 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x01\x7f\x01\x7f\x60\x00\x00\x02\x8a\x80\x80\x80\x00\x01\x01\x4d\x03\x6d\x65\x6d\x02\x00\x02\x03\x85\x80\x80\x80\x00\x04\x00\x00\x01\x01\x05\x83\x80\x80\x80\x00\x01\x00\x03\x07\xad\x80\x80\x80\x00\x04\x05\x72\x65\x61\x64\x31\x00\x00\x05\x72\x65\x61\x64\x32\x00\x01\x0b\x63\x6f\x70\x79\x2d\x31\x2d\x74\x6f\x2d\x32\x00\x02\x0b\x63\x6f\x70\x79\x2d\x32\x2d\x74\x6f\x2d\x31\x00\x03\x0a\xf0\x80\x80\x80\x00\x04\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x2d\x40\x01\x00\x0b\xa6\x80\x80\x80\x00\x01\x01\x7f\x41\x14\x21\x00\x03\x40\x20\x00\x41\x17\x46\x0d\x01\x20\x00\x20\x00\x2d\x00\x00\x3a\x40\x01\x00\x20\x00\x41\x01\x6a\x21\x00\x0c\x00\x0b\x0b\xa6\x80\x80\x80\x00\x01\x01\x7f\x41\x32\x21\x00\x03\x40\x20\x00\x41\x36\x46\x0d\x01\x20\x00\x20\x00\x2d\x40\x01\x00\x3a\x00\x00\x20\x00\x41\x01\x6a\x21\x00\x0c\x00\x0b\x0b\x0b\x96\x80\x80\x80\x00\x02\x00\x41\x14\x0b\x05\x01\x02\x03\x04\x05\x02\x01\x41\x32\x0b\x05\x0a\x0b\x0c\x0d\x0e", "store2.wast:6");

// store2.wast:6
let $2 = instance($$2);

// store2.wast:43
assert_return(() => call($2, "read2", [20]), "store2.wast:43", 0);

// store2.wast:44
assert_return(() => call($2, "read2", [21]), "store2.wast:44", 0);

// store2.wast:45
assert_return(() => call($2, "read2", [22]), "store2.wast:45", 0);

// store2.wast:46
assert_return(() => call($2, "read2", [23]), "store2.wast:46", 0);

// store2.wast:47
assert_return(() => call($2, "read2", [24]), "store2.wast:47", 0);

// store2.wast:48
run(() => call($2, "copy-1-to-2", []), "store2.wast:48");

// store2.wast:49
assert_return(() => call($2, "read2", [20]), "store2.wast:49", 1);

// store2.wast:50
assert_return(() => call($2, "read2", [21]), "store2.wast:50", 2);

// store2.wast:51
assert_return(() => call($2, "read2", [22]), "store2.wast:51", 3);

// store2.wast:52
assert_return(() => call($2, "read2", [23]), "store2.wast:52", 0);

// store2.wast:53
assert_return(() => call($2, "read2", [24]), "store2.wast:53", 0);

// store2.wast:55
assert_return(() => call($2, "read1", [50]), "store2.wast:55", 0);

// store2.wast:56
assert_return(() => call($2, "read1", [51]), "store2.wast:56", 0);

// store2.wast:57
assert_return(() => call($2, "read1", [52]), "store2.wast:57", 0);

// store2.wast:58
assert_return(() => call($2, "read1", [53]), "store2.wast:58", 0);

// store2.wast:59
assert_return(() => call($2, "read1", [54]), "store2.wast:59", 0);

// store2.wast:60
run(() => call($2, "copy-2-to-1", []), "store2.wast:60");

// store2.wast:61
assert_return(() => call($2, "read1", [50]), "store2.wast:61", 10);

// store2.wast:62
assert_return(() => call($2, "read1", [51]), "store2.wast:62", 11);

// store2.wast:63
assert_return(() => call($2, "read1", [52]), "store2.wast:63", 12);

// store2.wast:64
assert_return(() => call($2, "read1", [53]), "store2.wast:64", 13);

// store2.wast:65
assert_return(() => call($2, "read1", [54]), "store2.wast:65", 0);
reinitializeRegistry();
})();
