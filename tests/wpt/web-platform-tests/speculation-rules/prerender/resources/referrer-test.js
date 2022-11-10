async function referrer_test(expected, uid) {
  const bc = new PrerenderChannel('prerender-channel', uid);

  const gotMessage = new Promise(resolve => {
    bc.addEventListener('message', e => {
      resolve(e.data);
    }, {once: true});
  });

  // Start prerendering a page that will echo its referrer.
  startPrerendering(`resources/echo-referrer.py?uid=${uid}`);

  const result = await gotMessage;
  assert_equals(result.referrer, expected, 'referrer');
}
