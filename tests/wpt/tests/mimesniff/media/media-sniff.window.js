const tests = {
  "vectors": [
    "mp3-raw.mp3",
    "mp3-with-id3.mp3",
    "flac.flac",
    "ogg.ogg",
    "mp4.mp4",
    "wav.wav",
    "webm.webm"
  ],
  "contentTypes": [
    "",
    "bogus/mime",
    "application/octet-stream",
    "text/html",
    "audio/ogg; codec=vorbis",
    "application/pdf"
  ]
};

tests.vectors.forEach(vector => {
  tests.contentTypes.forEach(type => {
    async_test(t => {
      const element = document.createElement("audio");
      element.src = "resources/" + vector + "?pipe=header(Content-Type,"+type+")"

      element.addEventListener("error", t.unreached_func("No error expected frorm the HTMLMediaElement"));
      element.addEventListener("loadedmetadata", t.step_func_done());
      element.load();
    }, vector + " loads when served with Content-Type " + type);
  });
});
