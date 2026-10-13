(function wide_arithmetic_wast_js() {

// wide-arithmetic.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x91\x80\x80\x80\x00\x02\x60\x04\x7e\x7e\x7e\x7e\x02\x7e\x7e\x60\x02\x7e\x7e\x02\x7e\x7e\x03\x85\x80\x80\x80\x00\x04\x00\x00\x01\x01\x07\xbd\x80\x80\x80\x00\x04\x0a\x69\x36\x34\x2e\x61\x64\x64\x31\x32\x38\x00\x00\x0a\x69\x36\x34\x2e\x73\x75\x62\x31\x32\x38\x00\x01\x0e\x69\x36\x34\x2e\x6d\x75\x6c\x5f\x77\x69\x64\x65\x5f\x73\x00\x02\x0e\x69\x36\x34\x2e\x6d\x75\x6c\x5f\x77\x69\x64\x65\x5f\x75\x00\x03\x0a\xbd\x80\x80\x80\x00\x04\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\x20\x03\xfc\x13\x0b\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\x20\x03\xfc\x14\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x20\x01\xfc\x15\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x20\x01\xfc\x16\x0b", "wide-arithmetic.wast:1");

// wide-arithmetic.wast:1
let $1 = instance($$1);

// wide-arithmetic.wast:25
assert_return(() => call($1, "i64.add128", [0n, 0n, 0n, 0n]), "wide-arithmetic.wast:25", 0n, 0n);

// wide-arithmetic.wast:29
assert_return(() => call($1, "i64.add128", [0n, 1n, 1n, 0n]), "wide-arithmetic.wast:29", 1n, 1n);

// wide-arithmetic.wast:33
assert_return(() => call($1, "i64.add128", [1n, 0n, -1n, 0n]), "wide-arithmetic.wast:33", 0n, 1n);

// wide-arithmetic.wast:37
assert_return(() => call($1, "i64.add128", [1n, 1n, -1n, -1n]), "wide-arithmetic.wast:37", 0n, 1n);

// wide-arithmetic.wast:43
assert_return(() => call($1, "i64.sub128", [0n, 0n, 0n, 0n]), "wide-arithmetic.wast:43", 0n, 0n);

// wide-arithmetic.wast:47
assert_return(() => call($1, "i64.sub128", [0n, 0n, 1n, 0n]), "wide-arithmetic.wast:47", -1n, -1n);

// wide-arithmetic.wast:51
assert_return(() => call($1, "i64.sub128", [0n, 1n, 1n, 1n]), "wide-arithmetic.wast:51", -1n, -1n);

// wide-arithmetic.wast:55
assert_return(() => call($1, "i64.sub128", [0n, 0n, 1n, 1n]), "wide-arithmetic.wast:55", -1n, -2n);

// wide-arithmetic.wast:61
assert_return(() => call($1, "i64.mul_wide_s", [0n, 0n]), "wide-arithmetic.wast:61", 0n, 0n);

// wide-arithmetic.wast:63
assert_return(() => call($1, "i64.mul_wide_u", [0n, 0n]), "wide-arithmetic.wast:63", 0n, 0n);

// wide-arithmetic.wast:65
assert_return(() => call($1, "i64.mul_wide_s", [1n, 1n]), "wide-arithmetic.wast:65", 1n, 0n);

// wide-arithmetic.wast:67
assert_return(() => call($1, "i64.mul_wide_u", [1n, 1n]), "wide-arithmetic.wast:67", 1n, 0n);

// wide-arithmetic.wast:69
assert_return(() => call($1, "i64.mul_wide_s", [-1n, -1n]), "wide-arithmetic.wast:69", 1n, 0n);

// wide-arithmetic.wast:71
assert_return(() => call($1, "i64.mul_wide_s", [-1n, 1n]), "wide-arithmetic.wast:71", -1n, -1n);

// wide-arithmetic.wast:73
assert_return(() => call($1, "i64.mul_wide_u", [-1n, 1n]), "wide-arithmetic.wast:73", -1n, 0n);

// wide-arithmetic.wast:77
assert_return(() => call($1, "i64.add128", [-2_418_420_703_207_364_752n, -1n, -1n, -1n]), "wide-arithmetic.wast:77", -2_418_420_703_207_364_753n, -1n);

// wide-arithmetic.wast:81
assert_return(() => call($1, "i64.add128", [0n, 0n, -4_579_433_644_172_935_106n, -1n]), "wide-arithmetic.wast:81", -4_579_433_644_172_935_106n, -1n);

// wide-arithmetic.wast:85
assert_return(() => call($1, "i64.add128", [0n, 0n, 1n, -1n]), "wide-arithmetic.wast:85", 1n, -1n);

// wide-arithmetic.wast:89
assert_return(() => call($1, "i64.add128", [1n, 0n, 1n, 0n]), "wide-arithmetic.wast:89", 2n, 0n);

// wide-arithmetic.wast:93
assert_return(() => call($1, "i64.add128", [-1n, -1n, -1n, -1n]), "wide-arithmetic.wast:93", -2n, -1n);

// wide-arithmetic.wast:97
assert_return(() => call($1, "i64.add128", [0n, -1n, 1n, 0n]), "wide-arithmetic.wast:97", 1n, -1n);

// wide-arithmetic.wast:101
assert_return(() => call($1, "i64.add128", [0n, 0n, 0n, -1n]), "wide-arithmetic.wast:101", 0n, -1n);

// wide-arithmetic.wast:105
assert_return(() => call($1, "i64.add128", [1n, 0n, -1n, -1n]), "wide-arithmetic.wast:105", 0n, 0n);

// wide-arithmetic.wast:109
assert_return(() => call($1, "i64.add128", [0n, 6_184_727_276_166_606_191n, 0n, 1n]), "wide-arithmetic.wast:109", 0n, 6_184_727_276_166_606_192n);

// wide-arithmetic.wast:113
assert_return(() => call($1, "i64.add128", [-8_434_911_321_912_688_222n, -1n, 1n, -1n]), "wide-arithmetic.wast:113", -8_434_911_321_912_688_221n, -2n);

// wide-arithmetic.wast:117
assert_return(() => call($1, "i64.add128", [1n, -1n, 0n, -1n]), "wide-arithmetic.wast:117", 1n, -2n);

// wide-arithmetic.wast:121
assert_return(() => call($1, "i64.add128", [1n, -5_148_941_131_328_838_092n, 0n, 0n]), "wide-arithmetic.wast:121", 1n, -5_148_941_131_328_838_092n);

// wide-arithmetic.wast:125
assert_return(() => call($1, "i64.add128", [1n, 1n, 1n, 0n]), "wide-arithmetic.wast:125", 2n, 1n);

// wide-arithmetic.wast:129
assert_return(() => call($1, "i64.add128", [-1n, -1n, -3_636_740_005_180_858_631n, -1n]), "wide-arithmetic.wast:129", -3_636_740_005_180_858_632n, -1n);

// wide-arithmetic.wast:133
assert_return(() => call($1, "i64.add128", [-5_529_682_780_229_988_275n, -1n, 0n, 0n]), "wide-arithmetic.wast:133", -5_529_682_780_229_988_275n, -1n);

// wide-arithmetic.wast:137
assert_return(() => call($1, "i64.add128", [1n, -5_381_447_440_966_559_717n, 1_020_031_372_481_336_745n, 1n]), "wide-arithmetic.wast:137", 1_020_031_372_481_336_746n, -5_381_447_440_966_559_716n);

// wide-arithmetic.wast:141
assert_return(() => call($1, "i64.add128", [1n, 1n, 0n, 0n]), "wide-arithmetic.wast:141", 1n, 1n);

// wide-arithmetic.wast:145
assert_return(() => call($1, "i64.add128", [-9_133_888_546_939_907_356n, -1n, 1n, 1n]), "wide-arithmetic.wast:145", -9_133_888_546_939_907_355n, 0n);

// wide-arithmetic.wast:149
assert_return(() => call($1, "i64.add128", [-4_612_047_512_704_241_719n, -1n, 0n, -1n]), "wide-arithmetic.wast:149", -4_612_047_512_704_241_719n, -2n);

// wide-arithmetic.wast:153
assert_return(() => call($1, "i64.add128", [414_720_966_820_876_428n, -1n, 1n, 0n]), "wide-arithmetic.wast:153", 414_720_966_820_876_429n, -1n);

// wide-arithmetic.wast:160
assert_return(() => call($1, "i64.sub128", [0n, -2_459_085_471_354_756_766n, -9_151_153_060_221_070_927n, -1n]), "wide-arithmetic.wast:160", 9_151_153_060_221_070_927n, -2_459_085_471_354_756_766n);

// wide-arithmetic.wast:164
assert_return(() => call($1, "i64.sub128", [4_566_502_638_724_063_423n, -4_282_658_540_409_485_563n, -6_884_077_310_018_979_971n, -1n]), "wide-arithmetic.wast:164", -6_996_164_124_966_508_222n, -4_282_658_540_409_485_563n);

// wide-arithmetic.wast:168
assert_return(() => call($1, "i64.sub128", [1n, 3_118_380_319_444_903_041n, 0n, 3_283_115_686_417_695_443n]), "wide-arithmetic.wast:168", 1n, -164_735_366_972_792_402n);

// wide-arithmetic.wast:172
assert_return(() => call($1, "i64.sub128", [-7_208_415_241_680_161_810n, -1n, 1n, 0n]), "wide-arithmetic.wast:172", -7_208_415_241_680_161_811n, -1n);

// wide-arithmetic.wast:176
assert_return(() => call($1, "i64.sub128", [0n, 3_944_850_126_731_328_706n, 1n, 1n]), "wide-arithmetic.wast:176", -1n, 3_944_850_126_731_328_704n);

// wide-arithmetic.wast:180
assert_return(() => call($1, "i64.sub128", [1n, -1n, -1n, -1n]), "wide-arithmetic.wast:180", 2n, -1n);

// wide-arithmetic.wast:184
assert_return(() => call($1, "i64.sub128", [-1n, -1n, 4_855_833_073_346_115_923n, -6_826_437_637_438_999_645n]), "wide-arithmetic.wast:184", -4_855_833_073_346_115_924n, 6_826_437_637_438_999_644n);

// wide-arithmetic.wast:188
assert_return(() => call($1, "i64.sub128", [1n, 0n, -1n, -1n]), "wide-arithmetic.wast:188", 2n, 0n);

// wide-arithmetic.wast:192
assert_return(() => call($1, "i64.sub128", [1n, 0n, 1n, 0n]), "wide-arithmetic.wast:192", 0n, 0n);

// wide-arithmetic.wast:196
assert_return(() => call($1, "i64.sub128", [-1n, -1n, 0n, 0n]), "wide-arithmetic.wast:196", -1n, -1n);

// wide-arithmetic.wast:200
assert_return(() => call($1, "i64.sub128", [1n, -1n, -6_365_475_388_498_096_428n, -1n]), "wide-arithmetic.wast:200", 6_365_475_388_498_096_429n, -1n);

// wide-arithmetic.wast:204
assert_return(() => call($1, "i64.sub128", [6_804_238_617_560_992_346n, -1n, 0n, -1n]), "wide-arithmetic.wast:204", 6_804_238_617_560_992_346n, 0n);

// wide-arithmetic.wast:208
assert_return(() => call($1, "i64.sub128", [0n, 1n, 1n, -7_756_145_513_466_453_619n]), "wide-arithmetic.wast:208", -1n, 7_756_145_513_466_453_619n);

// wide-arithmetic.wast:212
assert_return(() => call($1, "i64.sub128", [1n, -1n, 1n, 1n]), "wide-arithmetic.wast:212", 0n, -2n);

// wide-arithmetic.wast:216
assert_return(() => call($1, "i64.sub128", [0n, 1n, 1n, 0n]), "wide-arithmetic.wast:216", -1n, 0n);

// wide-arithmetic.wast:220
assert_return(() => call($1, "i64.sub128", [1n, 5_602_881_641_763_648_953n, -2_110_589_244_314_239_080n, -1n]), "wide-arithmetic.wast:220", 2_110_589_244_314_239_081n, 5_602_881_641_763_648_953n);

// wide-arithmetic.wast:224
assert_return(() => call($1, "i64.sub128", [0n, 1n, -1n, -1n]), "wide-arithmetic.wast:224", 1n, 1n);

// wide-arithmetic.wast:228
assert_return(() => call($1, "i64.sub128", [0n, -1n, 3_553_816_990_259_121_806n, -2_105_235_417_856_431_622n]), "wide-arithmetic.wast:228", -3_553_816_990_259_121_806n, 2_105_235_417_856_431_620n);

// wide-arithmetic.wast:232
assert_return(() => call($1, "i64.sub128", [1_861_102_705_894_987_245n, 1n, 3_713_781_778_534_059_871n, 1n]), "wide-arithmetic.wast:232", -1_852_679_072_639_072_626n, -1n);

// wide-arithmetic.wast:236
assert_return(() => call($1, "i64.sub128", [0n, -1n, 1n, 1_832_524_486_821_761_762n]), "wide-arithmetic.wast:236", -1n, -1_832_524_486_821_761_764n);

// wide-arithmetic.wast:242
assert_return(() => call($1, "i64.mul_wide_s", [1n, 1n]), "wide-arithmetic.wast:242", 1n, 0n);

// wide-arithmetic.wast:244
assert_return(() => call($1, "i64.mul_wide_s", [0n, 6_287_758_211_025_156_705n]), "wide-arithmetic.wast:244", 0n, 0n);

// wide-arithmetic.wast:246
assert_return(() => call($1, "i64.mul_wide_s", [-6_643_537_319_803_451_357n, 1n]), "wide-arithmetic.wast:246", -6_643_537_319_803_451_357n, -1n);

// wide-arithmetic.wast:248
assert_return(() => call($1, "i64.mul_wide_s", [-2_483_565_146_858_803_428n, 0n]), "wide-arithmetic.wast:248", 0n, 0n);

// wide-arithmetic.wast:250
assert_return(() => call($1, "i64.mul_wide_s", [1n, 1n]), "wide-arithmetic.wast:250", 1n, 0n);

// wide-arithmetic.wast:252
assert_return(() => call($1, "i64.mul_wide_s", [-3_838_951_433_439_430_085n, 3_471_602_925_362_676_030n]), "wide-arithmetic.wast:252", 5_186_941_893_001_237_834n, -722_475_195_264_825_124n);

// wide-arithmetic.wast:254
assert_return(() => call($1, "i64.mul_wide_s", [-8_262_495_286_814_853_129n, 7_883_241_869_666_573_970n]), "wide-arithmetic.wast:254", -8_557_189_786_755_031_842n, -3_530_988_912_334_554_469n);

// wide-arithmetic.wast:256
assert_return(() => call($1, "i64.mul_wide_s", [4_278_371_902_407_959_701n, 1n]), "wide-arithmetic.wast:256", 4_278_371_902_407_959_701n, 0n);

// wide-arithmetic.wast:258
assert_return(() => call($1, "i64.mul_wide_s", [-8_852_706_149_487_089_182n, -1n]), "wide-arithmetic.wast:258", 8_852_706_149_487_089_182n, 0n);

// wide-arithmetic.wast:260
assert_return(() => call($1, "i64.mul_wide_s", [1n, -1n]), "wide-arithmetic.wast:260", -1n, -1n);

// wide-arithmetic.wast:262
assert_return(() => call($1, "i64.mul_wide_s", [-1n, -4_329_244_561_838_653_387n]), "wide-arithmetic.wast:262", 4_329_244_561_838_653_387n, 0n);

// wide-arithmetic.wast:264
assert_return(() => call($1, "i64.mul_wide_s", [-1n, -1n]), "wide-arithmetic.wast:264", 1n, 0n);

// wide-arithmetic.wast:266
assert_return(() => call($1, "i64.mul_wide_s", [697_896_157_315_764_057n, 1n]), "wide-arithmetic.wast:266", 697_896_157_315_764_057n, 0n);

// wide-arithmetic.wast:268
assert_return(() => call($1, "i64.mul_wide_s", [1n, 1n]), "wide-arithmetic.wast:268", 1n, 0n);

// wide-arithmetic.wast:270
assert_return(() => call($1, "i64.mul_wide_s", [-1n, 0n]), "wide-arithmetic.wast:270", 0n, 0n);

// wide-arithmetic.wast:272
assert_return(() => call($1, "i64.mul_wide_s", [0n, -3_769_664_482_072_947_073n]), "wide-arithmetic.wast:272", 0n, 0n);

// wide-arithmetic.wast:274
assert_return(() => call($1, "i64.mul_wide_s", [1n, 8_414_291_037_346_403_854n]), "wide-arithmetic.wast:274", 8_414_291_037_346_403_854n, 0n);

// wide-arithmetic.wast:276
assert_return(() => call($1, "i64.mul_wide_s", [1n, -1n]), "wide-arithmetic.wast:276", -1n, -1n);

// wide-arithmetic.wast:278
assert_return(() => call($1, "i64.mul_wide_s", [5_014_655_679_779_318_485n, -5_080_037_812_563_681_985n]), "wide-arithmetic.wast:278", 2_842_857_627_777_395_563n, -1_380_983_027_057_486_843n);

// wide-arithmetic.wast:280
assert_return(() => call($1, "i64.mul_wide_s", [0n, 1n]), "wide-arithmetic.wast:280", 0n, 0n);

// wide-arithmetic.wast:284
assert_return(() => call($1, "i64.mul_wide_u", [-4_734_436_040_338_162_711n, 0n]), "wide-arithmetic.wast:284", 0n, 0n);

// wide-arithmetic.wast:286
assert_return(() => call($1, "i64.mul_wide_u", [1n, 0n]), "wide-arithmetic.wast:286", 0n, 0n);

// wide-arithmetic.wast:288
assert_return(() => call($1, "i64.mul_wide_u", [3_270_597_527_173_764_279n, 6_636_648_075_495_406_358n]), "wide-arithmetic.wast:288", -5_430_303_818_902_260_550n, 1_176_674_035_141_685_826n);

// wide-arithmetic.wast:290
assert_return(() => call($1, "i64.mul_wide_u", [-7_771_814_344_630_108_151n, 1n]), "wide-arithmetic.wast:290", -7_771_814_344_630_108_151n, 0n);

// wide-arithmetic.wast:292
assert_return(() => call($1, "i64.mul_wide_u", [1n, 0n]), "wide-arithmetic.wast:292", 0n, 0n);

// wide-arithmetic.wast:294
assert_return(() => call($1, "i64.mul_wide_u", [1n, -7_864_138_787_704_962_081n]), "wide-arithmetic.wast:294", -7_864_138_787_704_962_081n, 0n);

// wide-arithmetic.wast:296
assert_return(() => call($1, "i64.mul_wide_u", [1n, 518_555_141_550_256_010n]), "wide-arithmetic.wast:296", 518_555_141_550_256_010n, 0n);

// wide-arithmetic.wast:298
assert_return(() => call($1, "i64.mul_wide_u", [1n, -1n]), "wide-arithmetic.wast:298", -1n, 0n);

// wide-arithmetic.wast:300
assert_return(() => call($1, "i64.mul_wide_u", [1_118_900_477_321_231_571n, -1n]), "wide-arithmetic.wast:300", -1_118_900_477_321_231_571n, 1_118_900_477_321_231_570n);

// wide-arithmetic.wast:302
assert_return(() => call($1, "i64.mul_wide_u", [-1n, 0n]), "wide-arithmetic.wast:302", 0n, 0n);

// wide-arithmetic.wast:304
assert_return(() => call($1, "i64.mul_wide_u", [-5_586_890_671_027_490_027n, 1n]), "wide-arithmetic.wast:304", -5_586_890_671_027_490_027n, 0n);

// wide-arithmetic.wast:306
assert_return(() => call($1, "i64.mul_wide_u", [0n, 3_603_850_799_751_152_505n]), "wide-arithmetic.wast:306", 0n, 0n);

// wide-arithmetic.wast:308
assert_return(() => call($1, "i64.mul_wide_u", [-1n, -1n]), "wide-arithmetic.wast:308", 1n, -2n);

// wide-arithmetic.wast:310
assert_return(() => call($1, "i64.mul_wide_u", [0n, 1n]), "wide-arithmetic.wast:310", 0n, 0n);

// wide-arithmetic.wast:312
assert_return(() => call($1, "i64.mul_wide_u", [-7_344_082_851_774_441_644n, 3_896_439_839_137_544_024n]), "wide-arithmetic.wast:312", 5_738_542_512_914_895_072n, 2_345_175_459_296_971_666n);

// wide-arithmetic.wast:314
assert_return(() => call($1, "i64.mul_wide_u", [0n, 0n]), "wide-arithmetic.wast:314", 0n, 0n);

// wide-arithmetic.wast:316
assert_return(() => call($1, "i64.mul_wide_u", [616_395_976_148_874_061n, 0n]), "wide-arithmetic.wast:316", 0n, 0n);

// wide-arithmetic.wast:318
assert_return(() => call($1, "i64.mul_wide_u", [2_810_729_703_362_889_816n, -1n]), "wide-arithmetic.wast:318", -2_810_729_703_362_889_816n, 2_810_729_703_362_889_815n);

// wide-arithmetic.wast:320
assert_return(() => call($1, "i64.mul_wide_u", [1n, -1n]), "wide-arithmetic.wast:320", -1n, 0n);

// wide-arithmetic.wast:322
assert_return(() => call($1, "i64.mul_wide_u", [1n, 0n]), "wide-arithmetic.wast:322", 0n, 0n);

// wide-arithmetic.wast:326
let $$2 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x11\x02\x60\x04\x7e\x7e\x7e\x7e\x02\x7e\x7e\x60\x02\x7e\x7e\x02\x7e\x7e\x03\x05\x04\x00\x00\x01\x01\x07\x3d\x04\x0a\x69\x36\x34\x2e\x61\x64\x64\x31\x32\x38\x00\x00\x0a\x69\x36\x34\x2e\x73\x75\x62\x31\x32\x38\x00\x01\x0e\x69\x36\x34\x2e\x6d\x75\x6c\x5f\x77\x69\x64\x65\x5f\x73\x00\x02\x0e\x69\x36\x34\x2e\x6d\x75\x6c\x5f\x77\x69\x64\x65\x5f\x75\x00\x03\x0a\x37\x04\x0e\x00\x20\x00\x20\x01\x20\x02\x20\x03\xfc\x93\x80\x00\x0b\x0d\x00\x20\x00\x20\x01\x20\x02\x20\x03\xfc\x94\x00\x0b\x0c\x00\x20\x00\x20\x01\xfc\x95\x80\x80\x80\x00\x0b\x0b\x00\x20\x00\x20\x01\xfc\x96\x80\x80\x00\x0b", "wide-arithmetic.wast:326");

// wide-arithmetic.wast:326
let $2 = instance($$2);

// wide-arithmetic.wast:385
assert_return(() => call($2, "i64.add128", [1n, 2n, 3n, 4n]), "wide-arithmetic.wast:385", 4n, 6n);

// wide-arithmetic.wast:389
assert_return(() => call($2, "i64.sub128", [2n, 5n, 1n, 2n]), "wide-arithmetic.wast:389", 1n, 3n);

// wide-arithmetic.wast:393
assert_return(() => call($2, "i64.mul_wide_s", [1n, -2n]), "wide-arithmetic.wast:393", -2n, -1n);

// wide-arithmetic.wast:395
assert_return(() => call($2, "i64.mul_wide_u", [3n, 2n]), "wide-arithmetic.wast:395", 6n, 0n);

// wide-arithmetic.wast:400
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x01\x60\x04\x7e\x7e\x7e\x7e\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\x20\x03\xfc\x13\x0b", "wide-arithmetic.wast:400");

// wide-arithmetic.wast:410
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x01\x60\x03\x7e\x7e\x7e\x02\x7e\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x13\x0b", "wide-arithmetic.wast:410");

// wide-arithmetic.wast:420
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x01\x60\x04\x7e\x7e\x7e\x7e\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x92\x80\x80\x80\x00\x01\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\x20\x03\xfc\x14\x0b", "wide-arithmetic.wast:420");

// wide-arithmetic.wast:430
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x01\x60\x03\x7e\x7e\x7e\x02\x7e\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x90\x80\x80\x80\x00\x01\x8a\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x14\x0b", "wide-arithmetic.wast:430");

// wide-arithmetic.wast:440
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x87\x80\x80\x80\x00\x01\x60\x02\x7e\x7e\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x00\x20\x00\x20\x01\xfc\x15\x0b", "wide-arithmetic.wast:440");

// wide-arithmetic.wast:448
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x87\x80\x80\x80\x00\x01\x60\x01\x7e\x02\x7e\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8c\x80\x80\x80\x00\x01\x86\x80\x80\x80\x00\x00\x20\x00\xfc\x15\x0b", "wide-arithmetic.wast:448");

// wide-arithmetic.wast:456
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x87\x80\x80\x80\x00\x01\x60\x02\x7e\x7e\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8e\x80\x80\x80\x00\x01\x88\x80\x80\x80\x00\x00\x20\x00\x20\x01\xfc\x16\x0b", "wide-arithmetic.wast:456");

// wide-arithmetic.wast:464
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x87\x80\x80\x80\x00\x01\x60\x01\x7e\x02\x7e\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x0a\x8c\x80\x80\x80\x00\x01\x86\x80\x80\x80\x00\x00\x20\x00\xfc\x16\x0b", "wide-arithmetic.wast:464");
reinitializeRegistry();
})();
