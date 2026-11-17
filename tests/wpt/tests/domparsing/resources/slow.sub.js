{
  const id = '{{GET[id]}}';
  const tag = '{{GET[tag]}}';
  if (id === '' || id === String(window.current_test_id)) {
    window.events?.push(tag || 'slow');
    window.on_slow?.();
  }
}
