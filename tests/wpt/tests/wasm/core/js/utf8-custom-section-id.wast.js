(function utf8_custom_section_id_wast_js() {

// utf8-custom-section-id.wast:6
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\x80", "utf8-custom-section-id.wast:6");

// utf8-custom-section-id.wast:16
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\x8f", "utf8-custom-section-id.wast:16");

// utf8-custom-section-id.wast:26
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\x90", "utf8-custom-section-id.wast:26");

// utf8-custom-section-id.wast:36
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\x9f", "utf8-custom-section-id.wast:36");

// utf8-custom-section-id.wast:46
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\xa0", "utf8-custom-section-id.wast:46");

// utf8-custom-section-id.wast:56
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\xbf", "utf8-custom-section-id.wast:56");

// utf8-custom-section-id.wast:68
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xc2\x80\x80", "utf8-custom-section-id.wast:68");

// utf8-custom-section-id.wast:78
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\xc2", "utf8-custom-section-id.wast:78");

// utf8-custom-section-id.wast:88
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xc2\x2e", "utf8-custom-section-id.wast:88");

// utf8-custom-section-id.wast:100
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xc0\x80", "utf8-custom-section-id.wast:100");

// utf8-custom-section-id.wast:110
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xc0\xbf", "utf8-custom-section-id.wast:110");

// utf8-custom-section-id.wast:120
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xc1\x80", "utf8-custom-section-id.wast:120");

// utf8-custom-section-id.wast:130
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xc1\xbf", "utf8-custom-section-id.wast:130");

// utf8-custom-section-id.wast:140
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xc2\x00", "utf8-custom-section-id.wast:140");

// utf8-custom-section-id.wast:150
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xc2\x7f", "utf8-custom-section-id.wast:150");

// utf8-custom-section-id.wast:160
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xc2\xc0", "utf8-custom-section-id.wast:160");

// utf8-custom-section-id.wast:170
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xc2\xfd", "utf8-custom-section-id.wast:170");

// utf8-custom-section-id.wast:180
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xdf\x00", "utf8-custom-section-id.wast:180");

// utf8-custom-section-id.wast:190
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xdf\x7f", "utf8-custom-section-id.wast:190");

// utf8-custom-section-id.wast:200
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xdf\xc0", "utf8-custom-section-id.wast:200");

// utf8-custom-section-id.wast:210
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xdf\xfd", "utf8-custom-section-id.wast:210");

// utf8-custom-section-id.wast:222
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xe1\x80\x80\x80", "utf8-custom-section-id.wast:222");

// utf8-custom-section-id.wast:232
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xe1\x80", "utf8-custom-section-id.wast:232");

// utf8-custom-section-id.wast:242
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe1\x80\x2e", "utf8-custom-section-id.wast:242");

// utf8-custom-section-id.wast:252
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\xe1", "utf8-custom-section-id.wast:252");

// utf8-custom-section-id.wast:262
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xe1\x2e", "utf8-custom-section-id.wast:262");

// utf8-custom-section-id.wast:274
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\x00\xa0", "utf8-custom-section-id.wast:274");

// utf8-custom-section-id.wast:284
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\x7f\xa0", "utf8-custom-section-id.wast:284");

// utf8-custom-section-id.wast:294
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\x80\x80", "utf8-custom-section-id.wast:294");

// utf8-custom-section-id.wast:304
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\x80\xa0", "utf8-custom-section-id.wast:304");

// utf8-custom-section-id.wast:314
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\x9f\xa0", "utf8-custom-section-id.wast:314");

// utf8-custom-section-id.wast:324
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\x9f\xbf", "utf8-custom-section-id.wast:324");

// utf8-custom-section-id.wast:334
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\xc0\xa0", "utf8-custom-section-id.wast:334");

// utf8-custom-section-id.wast:344
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\xfd\xa0", "utf8-custom-section-id.wast:344");

// utf8-custom-section-id.wast:354
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe1\x00\x80", "utf8-custom-section-id.wast:354");

// utf8-custom-section-id.wast:364
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe1\x7f\x80", "utf8-custom-section-id.wast:364");

// utf8-custom-section-id.wast:374
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe1\xc0\x80", "utf8-custom-section-id.wast:374");

// utf8-custom-section-id.wast:384
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe1\xfd\x80", "utf8-custom-section-id.wast:384");

// utf8-custom-section-id.wast:394
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xec\x00\x80", "utf8-custom-section-id.wast:394");

// utf8-custom-section-id.wast:404
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xec\x7f\x80", "utf8-custom-section-id.wast:404");

// utf8-custom-section-id.wast:414
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xec\xc0\x80", "utf8-custom-section-id.wast:414");

// utf8-custom-section-id.wast:424
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xec\xfd\x80", "utf8-custom-section-id.wast:424");

// utf8-custom-section-id.wast:434
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\x00\x80", "utf8-custom-section-id.wast:434");

// utf8-custom-section-id.wast:444
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\x7f\x80", "utf8-custom-section-id.wast:444");

// utf8-custom-section-id.wast:454
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\xa0\x80", "utf8-custom-section-id.wast:454");

// utf8-custom-section-id.wast:464
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\xa0\xbf", "utf8-custom-section-id.wast:464");

// utf8-custom-section-id.wast:474
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\xbf\x80", "utf8-custom-section-id.wast:474");

// utf8-custom-section-id.wast:484
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\xbf\xbf", "utf8-custom-section-id.wast:484");

// utf8-custom-section-id.wast:494
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\xc0\x80", "utf8-custom-section-id.wast:494");

// utf8-custom-section-id.wast:504
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\xfd\x80", "utf8-custom-section-id.wast:504");

// utf8-custom-section-id.wast:514
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xee\x00\x80", "utf8-custom-section-id.wast:514");

// utf8-custom-section-id.wast:524
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xee\x7f\x80", "utf8-custom-section-id.wast:524");

// utf8-custom-section-id.wast:534
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xee\xc0\x80", "utf8-custom-section-id.wast:534");

// utf8-custom-section-id.wast:544
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xee\xfd\x80", "utf8-custom-section-id.wast:544");

// utf8-custom-section-id.wast:554
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xef\x00\x80", "utf8-custom-section-id.wast:554");

// utf8-custom-section-id.wast:564
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xef\x7f\x80", "utf8-custom-section-id.wast:564");

// utf8-custom-section-id.wast:574
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xef\xc0\x80", "utf8-custom-section-id.wast:574");

// utf8-custom-section-id.wast:584
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xef\xfd\x80", "utf8-custom-section-id.wast:584");

// utf8-custom-section-id.wast:596
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\xa0\x00", "utf8-custom-section-id.wast:596");

// utf8-custom-section-id.wast:606
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\xa0\x7f", "utf8-custom-section-id.wast:606");

// utf8-custom-section-id.wast:616
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\xa0\xc0", "utf8-custom-section-id.wast:616");

// utf8-custom-section-id.wast:626
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe0\xa0\xfd", "utf8-custom-section-id.wast:626");

// utf8-custom-section-id.wast:636
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe1\x80\x00", "utf8-custom-section-id.wast:636");

// utf8-custom-section-id.wast:646
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe1\x80\x7f", "utf8-custom-section-id.wast:646");

// utf8-custom-section-id.wast:656
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe1\x80\xc0", "utf8-custom-section-id.wast:656");

// utf8-custom-section-id.wast:666
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xe1\x80\xfd", "utf8-custom-section-id.wast:666");

// utf8-custom-section-id.wast:676
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xec\x80\x00", "utf8-custom-section-id.wast:676");

// utf8-custom-section-id.wast:686
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xec\x80\x7f", "utf8-custom-section-id.wast:686");

// utf8-custom-section-id.wast:696
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xec\x80\xc0", "utf8-custom-section-id.wast:696");

// utf8-custom-section-id.wast:706
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xec\x80\xfd", "utf8-custom-section-id.wast:706");

// utf8-custom-section-id.wast:716
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\x80\x00", "utf8-custom-section-id.wast:716");

// utf8-custom-section-id.wast:726
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\x80\x7f", "utf8-custom-section-id.wast:726");

// utf8-custom-section-id.wast:736
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\x80\xc0", "utf8-custom-section-id.wast:736");

// utf8-custom-section-id.wast:746
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xed\x80\xfd", "utf8-custom-section-id.wast:746");

// utf8-custom-section-id.wast:756
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xee\x80\x00", "utf8-custom-section-id.wast:756");

// utf8-custom-section-id.wast:766
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xee\x80\x7f", "utf8-custom-section-id.wast:766");

// utf8-custom-section-id.wast:776
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xee\x80\xc0", "utf8-custom-section-id.wast:776");

// utf8-custom-section-id.wast:786
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xee\x80\xfd", "utf8-custom-section-id.wast:786");

// utf8-custom-section-id.wast:796
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xef\x80\x00", "utf8-custom-section-id.wast:796");

// utf8-custom-section-id.wast:806
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xef\x80\x7f", "utf8-custom-section-id.wast:806");

// utf8-custom-section-id.wast:816
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xef\x80\xc0", "utf8-custom-section-id.wast:816");

// utf8-custom-section-id.wast:826
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xef\x80\xfd", "utf8-custom-section-id.wast:826");

// utf8-custom-section-id.wast:838
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x06\x05\xf1\x80\x80\x80\x80", "utf8-custom-section-id.wast:838");

// utf8-custom-section-id.wast:848
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xf1\x80\x80", "utf8-custom-section-id.wast:848");

// utf8-custom-section-id.wast:858
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x80\x80\x23", "utf8-custom-section-id.wast:858");

// utf8-custom-section-id.wast:868
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xf1\x80", "utf8-custom-section-id.wast:868");

// utf8-custom-section-id.wast:878
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xf1\x80\x23", "utf8-custom-section-id.wast:878");

// utf8-custom-section-id.wast:888
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\xf1", "utf8-custom-section-id.wast:888");

// utf8-custom-section-id.wast:898
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xf1\x23", "utf8-custom-section-id.wast:898");

// utf8-custom-section-id.wast:910
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x00\x90\x90", "utf8-custom-section-id.wast:910");

// utf8-custom-section-id.wast:920
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x7f\x90\x90", "utf8-custom-section-id.wast:920");

// utf8-custom-section-id.wast:930
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x80\x80\x80", "utf8-custom-section-id.wast:930");

// utf8-custom-section-id.wast:940
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x80\x90\x90", "utf8-custom-section-id.wast:940");

// utf8-custom-section-id.wast:950
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x8f\x90\x90", "utf8-custom-section-id.wast:950");

// utf8-custom-section-id.wast:960
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x8f\xbf\xbf", "utf8-custom-section-id.wast:960");

// utf8-custom-section-id.wast:970
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\xc0\x90\x90", "utf8-custom-section-id.wast:970");

// utf8-custom-section-id.wast:980
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\xfd\x90\x90", "utf8-custom-section-id.wast:980");

// utf8-custom-section-id.wast:990
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x00\x80\x80", "utf8-custom-section-id.wast:990");

// utf8-custom-section-id.wast:1000
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x7f\x80\x80", "utf8-custom-section-id.wast:1000");

// utf8-custom-section-id.wast:1010
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\xc0\x80\x80", "utf8-custom-section-id.wast:1010");

// utf8-custom-section-id.wast:1020
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\xfd\x80\x80", "utf8-custom-section-id.wast:1020");

// utf8-custom-section-id.wast:1030
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x00\x80\x80", "utf8-custom-section-id.wast:1030");

// utf8-custom-section-id.wast:1040
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x7f\x80\x80", "utf8-custom-section-id.wast:1040");

// utf8-custom-section-id.wast:1050
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\xc0\x80\x80", "utf8-custom-section-id.wast:1050");

// utf8-custom-section-id.wast:1060
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\xfd\x80\x80", "utf8-custom-section-id.wast:1060");

// utf8-custom-section-id.wast:1070
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x00\x80\x80", "utf8-custom-section-id.wast:1070");

// utf8-custom-section-id.wast:1080
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x7f\x80\x80", "utf8-custom-section-id.wast:1080");

// utf8-custom-section-id.wast:1090
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x90\x80\x80", "utf8-custom-section-id.wast:1090");

// utf8-custom-section-id.wast:1100
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\xbf\x80\x80", "utf8-custom-section-id.wast:1100");

// utf8-custom-section-id.wast:1110
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\xc0\x80\x80", "utf8-custom-section-id.wast:1110");

// utf8-custom-section-id.wast:1120
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\xfd\x80\x80", "utf8-custom-section-id.wast:1120");

// utf8-custom-section-id.wast:1130
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf5\x80\x80\x80", "utf8-custom-section-id.wast:1130");

// utf8-custom-section-id.wast:1140
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf7\x80\x80\x80", "utf8-custom-section-id.wast:1140");

// utf8-custom-section-id.wast:1150
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf7\xbf\xbf\xbf", "utf8-custom-section-id.wast:1150");

// utf8-custom-section-id.wast:1162
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x90\x00\x90", "utf8-custom-section-id.wast:1162");

// utf8-custom-section-id.wast:1172
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x90\x7f\x90", "utf8-custom-section-id.wast:1172");

// utf8-custom-section-id.wast:1182
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x90\xc0\x90", "utf8-custom-section-id.wast:1182");

// utf8-custom-section-id.wast:1192
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x90\xfd\x90", "utf8-custom-section-id.wast:1192");

// utf8-custom-section-id.wast:1202
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x80\x00\x80", "utf8-custom-section-id.wast:1202");

// utf8-custom-section-id.wast:1212
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x80\x7f\x80", "utf8-custom-section-id.wast:1212");

// utf8-custom-section-id.wast:1222
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x80\xc0\x80", "utf8-custom-section-id.wast:1222");

// utf8-custom-section-id.wast:1232
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x80\xfd\x80", "utf8-custom-section-id.wast:1232");

// utf8-custom-section-id.wast:1242
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x80\x00\x80", "utf8-custom-section-id.wast:1242");

// utf8-custom-section-id.wast:1252
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x80\x7f\x80", "utf8-custom-section-id.wast:1252");

// utf8-custom-section-id.wast:1262
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x80\xc0\x80", "utf8-custom-section-id.wast:1262");

// utf8-custom-section-id.wast:1272
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x80\xfd\x80", "utf8-custom-section-id.wast:1272");

// utf8-custom-section-id.wast:1282
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x80\x00\x80", "utf8-custom-section-id.wast:1282");

// utf8-custom-section-id.wast:1292
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x80\x7f\x80", "utf8-custom-section-id.wast:1292");

// utf8-custom-section-id.wast:1302
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x80\xc0\x80", "utf8-custom-section-id.wast:1302");

// utf8-custom-section-id.wast:1312
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x80\xfd\x80", "utf8-custom-section-id.wast:1312");

// utf8-custom-section-id.wast:1324
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x90\x90\x00", "utf8-custom-section-id.wast:1324");

// utf8-custom-section-id.wast:1334
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x90\x90\x7f", "utf8-custom-section-id.wast:1334");

// utf8-custom-section-id.wast:1344
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x90\x90\xc0", "utf8-custom-section-id.wast:1344");

// utf8-custom-section-id.wast:1354
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf0\x90\x90\xfd", "utf8-custom-section-id.wast:1354");

// utf8-custom-section-id.wast:1364
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x80\x80\x00", "utf8-custom-section-id.wast:1364");

// utf8-custom-section-id.wast:1374
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x80\x80\x7f", "utf8-custom-section-id.wast:1374");

// utf8-custom-section-id.wast:1384
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x80\x80\xc0", "utf8-custom-section-id.wast:1384");

// utf8-custom-section-id.wast:1394
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf1\x80\x80\xfd", "utf8-custom-section-id.wast:1394");

// utf8-custom-section-id.wast:1404
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x80\x80\x00", "utf8-custom-section-id.wast:1404");

// utf8-custom-section-id.wast:1414
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x80\x80\x7f", "utf8-custom-section-id.wast:1414");

// utf8-custom-section-id.wast:1424
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x80\x80\xc0", "utf8-custom-section-id.wast:1424");

// utf8-custom-section-id.wast:1434
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf3\x80\x80\xfd", "utf8-custom-section-id.wast:1434");

// utf8-custom-section-id.wast:1444
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x80\x80\x00", "utf8-custom-section-id.wast:1444");

// utf8-custom-section-id.wast:1454
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x80\x80\x7f", "utf8-custom-section-id.wast:1454");

// utf8-custom-section-id.wast:1464
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x80\x80\xc0", "utf8-custom-section-id.wast:1464");

// utf8-custom-section-id.wast:1474
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf4\x80\x80\xfd", "utf8-custom-section-id.wast:1474");

// utf8-custom-section-id.wast:1486
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x07\x06\xf8\x80\x80\x80\x80\x80", "utf8-custom-section-id.wast:1486");

// utf8-custom-section-id.wast:1496
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf8\x80\x80\x80", "utf8-custom-section-id.wast:1496");

// utf8-custom-section-id.wast:1506
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x06\x05\xf8\x80\x80\x80\x23", "utf8-custom-section-id.wast:1506");

// utf8-custom-section-id.wast:1516
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xf8\x80\x80", "utf8-custom-section-id.wast:1516");

// utf8-custom-section-id.wast:1526
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xf8\x80\x80\x23", "utf8-custom-section-id.wast:1526");

// utf8-custom-section-id.wast:1536
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xf8\x80", "utf8-custom-section-id.wast:1536");

// utf8-custom-section-id.wast:1546
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xf8\x80\x23", "utf8-custom-section-id.wast:1546");

// utf8-custom-section-id.wast:1556
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\xf8", "utf8-custom-section-id.wast:1556");

// utf8-custom-section-id.wast:1566
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xf8\x23", "utf8-custom-section-id.wast:1566");

// utf8-custom-section-id.wast:1578
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x06\x05\xf8\x80\x80\x80\x80", "utf8-custom-section-id.wast:1578");

// utf8-custom-section-id.wast:1588
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x06\x05\xfb\xbf\xbf\xbf\xbf", "utf8-custom-section-id.wast:1588");

// utf8-custom-section-id.wast:1600
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x08\x07\xfc\x80\x80\x80\x80\x80\x80", "utf8-custom-section-id.wast:1600");

// utf8-custom-section-id.wast:1610
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x06\x05\xfc\x80\x80\x80\x80", "utf8-custom-section-id.wast:1610");

// utf8-custom-section-id.wast:1620
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x07\x06\xfc\x80\x80\x80\x80\x23", "utf8-custom-section-id.wast:1620");

// utf8-custom-section-id.wast:1630
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xfc\x80\x80\x80", "utf8-custom-section-id.wast:1630");

// utf8-custom-section-id.wast:1640
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x06\x05\xfc\x80\x80\x80\x23", "utf8-custom-section-id.wast:1640");

// utf8-custom-section-id.wast:1650
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xfc\x80\x80", "utf8-custom-section-id.wast:1650");

// utf8-custom-section-id.wast:1660
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xfc\x80\x80\x23", "utf8-custom-section-id.wast:1660");

// utf8-custom-section-id.wast:1670
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xfc\x80", "utf8-custom-section-id.wast:1670");

// utf8-custom-section-id.wast:1680
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x04\x03\xfc\x80\x23", "utf8-custom-section-id.wast:1680");

// utf8-custom-section-id.wast:1690
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\xfc", "utf8-custom-section-id.wast:1690");

// utf8-custom-section-id.wast:1700
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xfc\x23", "utf8-custom-section-id.wast:1700");

// utf8-custom-section-id.wast:1712
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x07\x06\xfc\x80\x80\x80\x80\x80", "utf8-custom-section-id.wast:1712");

// utf8-custom-section-id.wast:1722
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x07\x06\xfd\xbf\xbf\xbf\xbf\xbf", "utf8-custom-section-id.wast:1722");

// utf8-custom-section-id.wast:1734
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\xfe", "utf8-custom-section-id.wast:1734");

// utf8-custom-section-id.wast:1744
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x02\x01\xff", "utf8-custom-section-id.wast:1744");

// utf8-custom-section-id.wast:1754
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xfe\xff", "utf8-custom-section-id.wast:1754");

// utf8-custom-section-id.wast:1764
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\x00\x00\xfe\xff", "utf8-custom-section-id.wast:1764");

// utf8-custom-section-id.wast:1774
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x03\x02\xff\xfe", "utf8-custom-section-id.wast:1774");

// utf8-custom-section-id.wast:1784
assert_malformed("\x00\x61\x73\x6d\x01\x00\x00\x00\x00\x05\x04\xff\xfe\x00\x00", "utf8-custom-section-id.wast:1784");
reinitializeRegistry();
})();
