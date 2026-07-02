const SAME_ORIGIN = 'https://{{host}}:{{ports[h2][0]}}';
const RESOURCES_PATH = '/connection-allowlist/tentative/resources';
const SAME_ORIGIN_RESOURCES_URL = SAME_ORIGIN + RESOURCES_PATH;

function navigateToTestCase(type, allow, key) {
  const alt_host = '{{hosts[alt][]}}';
  const https_port = '{{ports[https][0]}}';
  const params = new URLSearchParams();
  params.set('type', type);
  params.set('allow', allow);
  params.set('key', key);
  params.set('alt_host', alt_host);
  params.set('https_port', https_port);
  const url = SAME_ORIGIN_RESOURCES_URL +
      '/early-hints-connection-allowlist-loader.h2.py?' + params.toString();
  window.location.replace(url);
}
