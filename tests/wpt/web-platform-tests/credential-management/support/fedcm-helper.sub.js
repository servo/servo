export const alt_manifest_origin = 'https://{{hosts[alt][]}}:{{ports[https][0]}}';

// Set the identity provider cookie.
export function set_fedcm_cookie(host) {
  return new Promise(resolve => {
    if (host == undefined) {
      document.cookie = 'cookie=1; SameSite=Strict; Path=/credential-management/support; Secure';
      resolve();
    } else {
      let popup_window = window.open(host + '/credential-management/support/set_cookie');

      const popup_message_handler = (event) => {
        if (event.origin == host) {
          popup_window.close();
          window.removeEventListener('message', popup_message_handler);
          resolve();
        }
      };

      window.addEventListener('message', popup_message_handler);
    }
  });
}

// Set the alternate identity provider cookie.
export function set_alt_fedcm_cookie() {
  return set_fedcm_cookie(alt_manifest_origin);
}

// Returns FedCM CredentialRequestOptions for which navigator.credentials.get()
// succeeds.
export function default_request_options(manifest_filename) {
  if (manifest_filename === undefined) {
    manifest_filename = "manifest.py";
  }
  const manifest_path = `https://{{host}}:{{ports[https][0]}}/\
credential-management/support/fedcm/${manifest_filename}`;
  return {
    identity: {
      providers: [{
        configURL: manifest_path,
        clientId: '1',
        nonce: '2',
      }]
    }
  };
}

// Returns alternate FedCM CredentialRequestOptions for which navigator.credentials.get()
// succeeds.
export function default_alt_request_options(manifest_filename) {
  if (manifest_filename === undefined) {
    manifest_filename = "manifest.py";
  }
  const manifest_path = `${alt_manifest_origin}/\
credential-management/support/fedcm/${manifest_filename}`;
  return {
    identity: {
      providers: [{
        configURL: manifest_path,
        clientId: '1',
        nonce: '2',
      }]
    }
  };
}

// Test wrapper which does FedCM-specific setup.
export function fedcm_test(test_func, test_name) {
  promise_test(async t => {
    await set_fedcm_cookie();
    await set_alt_fedcm_cookie();
    await test_func(t);
  }, test_name);
}

function select_manifest_impl(manifest_url) {
  const url_query = (manifest_url === undefined)
      ? '' : '?manifest_url=${manifest_url}';

  return new Promise(resolve => {
    const img = document.createElement('img');
    img.src = 'support/fedcm/select_manifest_in_root_manifest.py?${url_query}';
    img.addEventListener('error', resolve);
    document.body.appendChild(img);
  });
}

// Sets the manifest returned by the next fetch of /.well-known/web_identity
// select_manifest() only affects the next fetch and not any subsequent fetches
// (ex second next fetch).
export function select_manifest(test, test_options) {
  // Add cleanup in case that /.well-known/web_identity is not fetched at all.
  test.add_cleanup(async () => {
    await select_manifest_impl();
  });
  const manifest_url = test_options.identity.providers[0].configURL;
  return select_manifest_impl(manifest_url);
}
