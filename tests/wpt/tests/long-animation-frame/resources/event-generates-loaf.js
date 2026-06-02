(() => {
  const xhr = new XMLHttpRequest();
  xhr.open('GET', '/common/dummy.xml');
  xhr.addEventListener('load', () => {
    generate_loaf_now();
  });
  xhr.send();
})();
