self.onmessage = function(e) {
  const offscreen = e.data.canvas;
  offscreen_ctx = offscreen.getContext('2d');

  offscreen_ctx.font = '25px serif';
  offscreen_ctx.lang = 'zh-CN';
  offscreen_ctx.fillText('今骨直', 5, 50);

  // Draw a single pixel, used to detect that the worker has completed.
  offscreen_ctx.fillStyle = '#0f0';
  offscreen_ctx.fillRect(0, 0, 1, 1);
}
