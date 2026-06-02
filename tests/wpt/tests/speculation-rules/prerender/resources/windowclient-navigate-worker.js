importScripts("/speculation-rules/prerender/resources/utils.js");

const params = new URLSearchParams(location.search);
const uid = params.get('uid');

const bc = new PrerenderChannel('navigation-channel', uid);

bc.onmessage = async e => {
  const data = JSON.parse(e.data);
  const navigationUrl = data.navigationUrl;
  const clientUrl = data.clientUrl;
  const respondTo = data.respondTo;

  const clients = await self.clients.matchAll();
  const client = clients.find(c => c.url == clientUrl);
  if (!client) {
    const bc = new PrerenderChannel(respondTo, uid);
    bc.postMessage('Client was not found');
    bc.close();
    return;
  }

  let result;
  try {
    await client.navigate(navigationUrl);
    result = 'navigate() succeeded';
  } catch (e) {
    if (e instanceof TypeError) {
      result = 'navigate() failed with TypeError';
    } else {
      result = 'navigate() failed with unknown error';
    }
  } finally {
    const bc = new PrerenderChannel(respondTo, uid);
    bc.postMessage(result);
    bc.close();
  }
};
