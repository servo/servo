// note, this template substitution is XSS, but no way to avoid it in this framework
var expected_logs = {{GET[logs]}};
var timeout = "{{GET[timeout]}}";
if (timeout == "") {
  timeout = 2;
}

if (expected_logs.length == 0) {
    function log_assert(msg) {
        test(function () { assert_unreached(msg) });
    }
} else {
    var t_log = async_test('Expecting logs: {{GET[logs]}}');
    step_timeout(function() {
      if(t_log.phase != t_log.phases.COMPLETE){
        t_log.step(function () { assert_unreached('Logging timeout, expected logs ' + expected_logs + ' not sent.') });
        t_log.done();
      }
    }, timeout * 1000);
    function log(msg) {
        //cons/**/ole.log(msg);
        t_log.step(function () {
            if (msg.match(/^FAIL/i)) {
                assert_unreached(msg);
                t_log.done();
            }
            for (var i = 0; i < expected_logs.length; i++) {
                if (expected_logs[i] == msg) {
                    assert_equals(expected_logs[i], msg);
                    expected_logs.splice(i, 1);
                    if (expected_logs.length == 0) {
                        t_log.done();
                    }
                    return;
                 }
             }
             assert_unreached('unexpected log: ' + msg);
             t_log.done();
        });
    }
}
