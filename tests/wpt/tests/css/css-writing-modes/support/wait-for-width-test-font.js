// Avoid fallback-font first layout for deterministic reftest snapshots.
document.fonts.load('72px WidthTest').then(() => {
  document.querySelector('.test').style.display = 'block';
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      document.documentElement.classList.remove('reftest-wait');
    });
  });
});
