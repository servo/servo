// note, this template substitution is XSS, but no way to avoid it in this framework
var expected_alerts = {{GET[alerts]}};
var timeout= "{{GET[timeout]}}";
if (timeout == "") {
  timeout = 2;
}

if(expected_alerts.length == 0) {
  function alert_assert(msg) {
   test(function () { assert_unreached(msg) });
 }
} else {
 var t_alert = async_test('Expecting alerts: {{GET[alerts]}}');
 step_timeout(function() {
   if(t_alert.phase != t_alert.phases.COMPLETE) {
     t_alert.step(function() { assert_unreached('Alert timeout, expected alerts ' + expected_alerts  + ' not fired.') });
     t_alert.done();
    }
 }, timeout * 1000);
 var alert_assert = function (msg) {
     t_alert.step(function () {
         if(msg && msg instanceof Error) {
             msg = msg.message;
         }
         if (msg && msg.match(/^FAIL/i)) {
             assert_unreached(msg);
             t_alert.done();
         }
         for (var i = 0; i < expected_alerts.length; i++) {
             if (expected_alerts[i] == msg) {
                 assert_equals(expected_alerts[i], msg);
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
 }.bind(this);
}
