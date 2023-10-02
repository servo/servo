(() => {
  const xhr = new XMLHttpRequest();
  xhr.open('GET', '/common/dummy.xml');
  xhr.addEventListener('load', () => {
    const deadline = performance.now() + 360;
    while (performance.now() < deadline) {
    }
  });
  xhr.send();
})();
