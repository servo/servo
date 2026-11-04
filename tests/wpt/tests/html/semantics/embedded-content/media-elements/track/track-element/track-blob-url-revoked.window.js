// META: title=track element with a blob: URL that is revoked

const vttText = "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nfoo\n";

function createTrack(t, blobUrl) {
  const video = document.createElement("video");
  const track = document.createElement("track");
  t.add_cleanup(() => video.remove());
  track.src = blobUrl;
  video.appendChild(track);
  document.body.appendChild(video);
  // Setting the mode starts the track processing model.
  track.track.mode = "hidden";
  return track;
}

function assertCueLoaded(track) {
  assert_equals(track.readyState, HTMLTrackElement.LOADED);
  const cues = track.track.cues;
  assert_equals(cues.length, 1);
  assert_equals(cues[0].startTime, 1);
  assert_equals(cues[0].endTime, 2);
  assert_equals(cues[0].text, "foo");
}

async_test(t => {
  const blobUrl = URL.createObjectURL(new Blob([vttText], { type: "text/vtt" }));
  t.add_cleanup(() => URL.revokeObjectURL(blobUrl));
  const track = createTrack(t, blobUrl);
  track.onload = t.step_func_done(() => assertCueLoaded(track));
  track.onerror = t.unreached_func("error event fired");
}, "track loads from a blob: URL");

async_test(t => {
  const blobUrl = URL.createObjectURL(new Blob([vttText], { type: "text/vtt" }));
  const track = createTrack(t, blobUrl);
  track.onload = t.step_func_done(() => assertCueLoaded(track));
  track.onerror = t.unreached_func("error event fired");
  // The blob URL entry is resolved when the src attribute is parsed, so
  // revoking it afterwards must not affect the load.
  URL.revokeObjectURL(blobUrl);
}, "track loads from a blob: URL revoked after src is set");

async_test(t => {
  const blobUrl = URL.createObjectURL(new Blob([vttText], { type: "text/vtt" }));
  // Revoked before the src attribute is set, so the URL no longer resolves.
  URL.revokeObjectURL(blobUrl);
  const track = createTrack(t, blobUrl);
  track.onload = t.unreached_func("load event fired");
  track.onerror = t.step_func_done(() => {
    assert_equals(track.readyState, HTMLTrackElement.ERROR);
  });
}, "track fails to load from a blob: URL revoked before src is set");
