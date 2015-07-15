// note, this template substitution is XSS, but no way to avoid it in this framework
var expected_alerts = {{GET[alerts]}};

if(expected_alerts.length == 0) {
  function alert_assert(msg) {
   test(function () { assert_unreached(msg) });
 }
} else {
 var t_alert = async_test('Expecting alerts: {{GET[alerts]}}');
 function alert_assert(msg) {
     t_alert.step(function () {
         if (msg.match(/^FAIL/i)) {
             assert_unreached(msg);
             t_alert.done();
         }
         for (var i = 0; i < expected_alerts.length; i++) {
             if (expected_alerts[i] == msg) {
                 assert_true(expected_alerts[i] == msg);
                 expected_alerts.splice(i, 1);
                 if (expected_alerts.length == 0) {
                     t_alert.done();
                 }
                 return;
             }
         }
         assert_unreached('unexpected alert: ' + msg);
         t_log.done();
     });
 }
}
