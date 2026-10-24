// META: title=media element with a blob: URL that is revoked

async function createBlobUrl() {
  const response = await fetch("/media/sound_0.mp3");
  const buffer = await response.arrayBuffer();
  return URL.createObjectURL(new Blob([buffer], { type: "audio/mpeg" }));
}

function createAudio(t) {
  const audio = document.createElement("audio");
  t.add_cleanup(() => audio.remove());
  document.body.appendChild(audio);
  return audio;
}

promise_test(async t => {
  const blobUrl = await createBlobUrl();
  t.add_cleanup(() => URL.revokeObjectURL(blobUrl));
  const audio = createAudio(t);
  const watcher = new EventWatcher(t, audio, ["loadeddata", "error"]);
  audio.src = blobUrl;
  await watcher.wait_for("loadeddata");
}, "media loads from a blob: URL");

promise_test(async t => {
  const blobUrl = await createBlobUrl();
  const audio = createAudio(t);
  const watcher = new EventWatcher(t, audio, ["loadeddata", "error"]);
  audio.src = blobUrl;
  // The fetch starts from a microtask queued when src was set,
  // so this microtask runs after the request has been created.
  queueMicrotask(() => URL.revokeObjectURL(blobUrl));
  await watcher.wait_for("loadeddata");
}, "media loads from a blob: URL revoked after the fetch started");

promise_test(async t => {
  const blobUrl = await createBlobUrl();
  // Revoked before the resource selection algorithm ran, so the URL no longer resolves.
  URL.revokeObjectURL(blobUrl);
  const audio = createAudio(t);
  const watcher = new EventWatcher(t, audio, ["loadeddata", "error"]);
  audio.src = blobUrl;
  await watcher.wait_for("error");
  assert_equals(audio.error.code, MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED);
}, "media fails to load from a blob: URL revoked before the fetch started");
