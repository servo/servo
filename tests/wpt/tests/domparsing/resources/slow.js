{
  const params = new URL(document.currentScript.src).searchParams;
  const id = params.get('id');
  if (id === null || id === String(window.current_test_id)) {
    window.events?.push(params.get('tag') ?? 'slow');
    window.on_slow?.();
  }
}
