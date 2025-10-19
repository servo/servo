(function load1_wast_js() {

// load1.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x86\x80\x80\x80\x00\x01\x60\x01\x7f\x01\x7f\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x00\x02\x07\x8e\x80\x80\x80\x00\x02\x03\x6d\x65\x6d\x02\x00\x04\x72\x65\x61\x64\x00\x00\x0a\x8d\x80\x80\x80\x00\x01\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b", "load1.wast:1");
let $M = $$1;

// load1.wast:1
let $1 = instance($M);
let M = $1;

// load1.wast:8
register("M", $1)

// load1.wast:10
let $$2 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x86\x80\x80\x80\x00\x01\x60\x01\x7f\x01\x7f\x02\x8a\x80\x80\x80\x00\x01\x01\x4d\x03\x6d\x65\x6d\x02\x00\x02\x03\x83\x80\x80\x80\x00\x02\x00\x00\x05\x83\x80\x80\x80\x00\x01\x00\x03\x07\x91\x80\x80\x80\x00\x02\x05\x72\x65\x61\x64\x31\x00\x00\x05\x72\x65\x61\x64\x32\x00\x01\x0a\x9a\x80\x80\x80\x00\x02\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x2d\x40\x01\x00\x0b\x0b\x96\x80\x80\x80\x00\x02\x00\x41\x14\x0b\x05\x01\x02\x03\x04\x05\x02\x01\x41\x32\x0b\x05\x0a\x0b\x0c\x0d\x0e", "load1.wast:10");

// load1.wast:10
let $2 = instance($$2);

// load1.wast:25
assert_return(() => call(M, "read", [20]), "load1.wast:25", 1);

// load1.wast:26
assert_return(() => call(M, "read", [21]), "load1.wast:26", 2);

// load1.wast:27
assert_return(() => call(M, "read", [22]), "load1.wast:27", 3);

// load1.wast:28
assert_return(() => call(M, "read", [23]), "load1.wast:28", 4);

// load1.wast:29
assert_return(() => call(M, "read", [24]), "load1.wast:29", 5);

// load1.wast:31
assert_return(() => call($2, "read1", [20]), "load1.wast:31", 1);

// load1.wast:32
assert_return(() => call($2, "read1", [21]), "load1.wast:32", 2);

// load1.wast:33
assert_return(() => call($2, "read1", [22]), "load1.wast:33", 3);

// load1.wast:34
assert_return(() => call($2, "read1", [23]), "load1.wast:34", 4);

// load1.wast:35
assert_return(() => call($2, "read1", [24]), "load1.wast:35", 5);

// load1.wast:37
assert_return(() => call($2, "read2", [50]), "load1.wast:37", 10);

// load1.wast:38
assert_return(() => call($2, "read2", [51]), "load1.wast:38", 11);

// load1.wast:39
assert_return(() => call($2, "read2", [52]), "load1.wast:39", 12);

// load1.wast:40
assert_return(() => call($2, "read2", [53]), "load1.wast:40", 13);

// load1.wast:41
assert_return(() => call($2, "read2", [54]), "load1.wast:41", 14);
reinitializeRegistry();
})();
