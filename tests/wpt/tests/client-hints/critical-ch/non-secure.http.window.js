// META: script=resources/util.js

async_test(make_message_test(ECHO_URL, "FAIL"), "Critical-CH navigation non-secure")
async_test(make_message_test(ECHO_URL+"?multiple=true", "FAIL"), "Critical-CH w/ multiple headers and navigation non-secure")
