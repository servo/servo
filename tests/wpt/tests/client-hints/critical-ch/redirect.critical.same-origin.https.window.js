// META: script=resources/util.js

async_test(make_message_test(REDIRECT_URL+"?critical=true&location=/client-hints/critical-ch/"+ECHO_URL, "FAIL"), "Critical-CH critical redirect")
