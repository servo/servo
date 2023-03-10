// META: script=resources/util.js
// META: script=/common/utils.js

async_test(make_message_test(ECHO_URL+"?multiple=true&token="+token(), "2"), "Critical-CH w/ multiple headers and navigation restart")
