// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=resources/webtransport-idlharness.js

'use strict';

idl_test(
  ['webtransport'],
  ['webidl', 'streams'],
  idl_array => {
    idl_array.set_test_filter(is_webtransport_writer_test);
  }
);
