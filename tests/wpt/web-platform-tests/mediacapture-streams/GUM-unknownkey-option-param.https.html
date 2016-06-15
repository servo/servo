<!doctype html>
<html>
<head>
<title>getUserMedia({doesnotexist:true}) aborts with NOT_SUPPORTED_ERR</title>
<link rel="author" title="Dominique Hazael-Massieux" href="mailto:dom@w3.org"/>
<link rel="help" href="http://dev.w3.org/2011/webrtc/editor/getusermedia.html#widl-NavigatorUserMedia-getUserMedia-void-MediaStreamConstraints-constraints-NavigatorUserMediaSuccessCallback-successCallback-NavigatorUserMediaErrorCallback-errorCallback">
</head>
<body>
<h1 class="instructions">Description</h1>
<p class="instructions">This test checks that getUserMedia with an unknown value
in the options parameter raises a NOT_SUPPORTED_ERR exception.</p>

<div id='log'></div>
<script src=/resources/testharness.js></script>
<script src=/resources/testharnessreport.js></script>
<script src="/common/vendor-prefix.js" data-prefixed-objects='[{"ancestors":["navigator"], "name":"getUserMedia"}]'></script>
<script>
test(function () {
  // TODO This is no longer what's in the spec, see https://www.w3.org/Bugs/Public/show_bug.cgi?id=22211
  assert_throws(
      "NOT_SUPPORTED_ERR",
      function () {
        navigator.getUserMedia(
            {doesnotexist:true},
            t.step_func(function (stream) {
              assert_unreached("This should never be triggered since the constraints parameter is unrecognized");
            }), t.step_func(function (error) {
              assert_unreached("This should never be triggered since the constraints parameter is unrecognized");
            }));
      }
  )
  }
);

</script>
</body>
</html>
