/**
 * Promise based helper function who's return promise will resolve
 * once the iframe src has been loaded
 * @param {string} url the url to set the iframe src
 * @param {test} t a test object to add a cleanup function to
 * @return {Promise} when resolved, will return the iframe
 */
self.createIframe = (url, t) => new Promise(resolve => {
  const iframe = document.createElement('iframe');
  iframe.addEventListener('load', () => {resolve(iframe);}, {once: true});
  iframe.src = url;
  document.documentElement.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());
});

/**
 * Function that will return a promise that resolves when a message event
 * is fired. Returns a promise that resolves to the message that was received
 */
self.waitForMessage = () => new Promise(resolve => {
  window.addEventListener('message', event => {
    resolve(event.data);
  }, {once: true});
});
