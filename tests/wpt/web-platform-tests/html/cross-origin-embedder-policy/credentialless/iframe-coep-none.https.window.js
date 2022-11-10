// META: variant=?1-4
// META: variant=?5-last
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js
// META: script=./resources/iframeTest.js
// META: script=/common/subset-tests.js

const parent_coep_none = newWindow(coep_none);
subsetTest(iframeTest, "COEP:none embeds same-origin COEP:none",
  parent_coep_none, same_origin, coep_none, EXPECT_LOAD);
subsetTest(iframeTest, "COEP:none embeds cross-origin COEP:none",
  parent_coep_none, cross_origin, coep_none, EXPECT_LOAD);
subsetTest(iframeTest, "COEP:none embeds same-origin COEP:credentialless",
  parent_coep_none, same_origin, coep_credentialless, EXPECT_LOAD);
subsetTest(iframeTest, "COEP:none embeds cross-origin COEP:credentialless",
  parent_coep_none, cross_origin, coep_credentialless, EXPECT_LOAD);
subsetTest(iframeTest, "COEP:none embeds same-origin COEP:require-corp",
  parent_coep_none, same_origin, coep_require_corp, EXPECT_LOAD);
subsetTest(iframeTest, "COEP:none embeds cross-origin COEP:require-corp",
  parent_coep_none, cross_origin, coep_require_corp, EXPECT_LOAD);
