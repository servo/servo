// META: script=resources/util.js

async_test(make_message_test(ECHO_URL+"?mismatch=true", "FAIL"), "Critical-CH Mis-matched hints")
async_test(make_message_test(ECHO_URL+"?multiple=true&mismatch=true", "FAIL"), "Critical-CH w/ multiple headers and Mis-matched hints")
