<!doctype html>
<html>
<head>
<title>Adding a track to a MediaStream</title>
<link rel="author" title="Dominique Hazael-Massieux" href="mailto:dom@w3.org"/>
<link rel="help" href="http://dev.w3.org/2011/webrtc/editor/getusermedia.html#widl-MediaStreamTrackList-add-void-MediaStreamTrack-track">
<link rel="help" href="http://dev.w3.org/2011/webrtc/editor/getusermedia.html#event-mediastream-addtrack">
</head>
<body>
<p class="instructions">When prompted, accept to share your audio stream, then your video stream.</p>
<h1 class="instructions">Description</h1>
<p class="instructions">This test checks that adding a track to a MediaStream works as expected.</p>

<div id='log'></div>
<script src=/resources/testharness.js></script>
<script src=/resources/testharnessreport.js></script>
<script src="/common/vendor-prefix.js" data-prefixed-objects='[{"ancestors":["navigator"], "name":"getUserMedia"}]'></script>
<script>
var t = async_test("Tests that adding a track to a MediaStream works as expected", {timeout: 20000}); // longer timeout since requires double user interaction
t.step(function () {
  var audio, video;

  navigator.getUserMedia({audio: true}, gotAudio, function(error) {});
  function gotAudio(stream) {
    audio = stream;
    navigator.getUserMedia({video: true}, gotVideo, function(error) {});
  }

  function gotVideo(stream) {
    video = stream;
    t.step(function () {
       assert_equals(video.getAudioTracks().length, 0, "video mediastream starts with no audio track");
       video.addTrack(audio.getAudioTracks()[0]);
       assert_equals(video.getAudioTracks().length, 1, "video mediastream has now one audio track");
       video.addTrack(audio.getAudioTracks()[0]);
       assert_equals(video.getAudioTracks().length, 1, "video mediastream still has one audio track"); // If track is already in stream's track set, then abort these steps.

    });
    audio.onaddtrack = t.step_func(function () {
       assert_unreached("onaddtrack is not fired when the script directly modified the track of a mediastream");
    });
    t.step(function () {
       assert_equals(audio.getVideoTracks().length, 0, "audio mediastream starts with no video track");
       audio.addTrack(video.getVideoTracks()[0]);
       assert_equals(audio.getVideoTracks().length, 1, "audio mediastream now has one video track");
    });
    t.step(function () {
       t.done();
    });
  }
});
</script>
</body>
</html>
