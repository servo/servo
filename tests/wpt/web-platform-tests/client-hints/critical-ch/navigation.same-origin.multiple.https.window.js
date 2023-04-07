// META: script=resources/util.js

async_test(make_message_test(ECHO_URL+"?multiple=true", "PASS"), "Critical-CH w/ multiple headers and navigation")
