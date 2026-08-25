// META: script=resources/util.js
// META: script=/common/get-host-info.sub.js

async_test(make_message_test(REDIRECT_URL+"?critical=true&location="+get_host_info().HTTPS_REMOTE_ORIGIN+"/client-hints/critical-ch/"+ECHO_URL+"?multiple=true", "PASS"), "Critical-CH w/ multiple headers cross-origin critical redirect")
