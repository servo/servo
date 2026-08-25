self.onmessage = function(e) {
  const offscreen = e.data.canvas;
  offscreen_ctx = offscreen.getContext('2d');

  let test_font = new FontFace(
    // Lato-Medium is a font with language specific ligatures.
    "Lato-Medium",
    "url(/fonts/Lato-Medium.ttf)"
  );

  test_font.load().then((font) => {
    self.fonts.add(font);
    offscreen_ctx.font = '25px Lato-Medium';
    offscreen_ctx.fillText('fi', 5, 50);

    // Draw a single pixel, used to detect that the worker has completed.
    offscreen_ctx.fillStyle = '#0f0';
    offscreen_ctx.fillRect(0, 0, 1, 1);
  });
}
