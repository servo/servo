export const manifest_origin = "https://{{host}}:{{ports[https][0]}}";
export const alt_manifest_origin = 'https://{{hosts[alt][]}}:{{ports[https][0]}}';

export function open_and_wait_for_popup(origin, path) {
  return new Promise(resolve => {
    let popup_window = window.open(origin + path);

    // We rely on the popup page to send us a message when done.
    const popup_message_handler = (event) => {
      if (event.origin == origin) {
        popup_window.close();
        window.removeEventListener('message', popup_message_handler);
        resolve();
      }
    };

    window.addEventListener('message', popup_message_handler);
  });
}

// Set the identity provider cookie.
export function set_fedcm_cookie(host) {
  if (host == undefined) {
    document.cookie = 'cookie=1; SameSite=Strict; Path=/credential-management/support; Secure';
    return Promise.resolve();
  } else {
    return open_and_wait_for_popup(host, '/credential-management/support/set_cookie');
  }
}

// Set the alternate identity provider cookie.
export function set_alt_fedcm_cookie() {
  return set_fedcm_cookie(alt_manifest_origin);
}

export function mark_signed_in(origin = manifest_origin) {
  return open_and_wait_for_popup(origin, '/credential-management/support/mark_signedin');
}

export function mark_signed_out(origin = manifest_origin) {
  return open_and_wait_for_popup(origin, '/credential-management/support/mark_signedout');
}

// Returns FedCM CredentialRequestOptions for which navigator.credentials.get()
// succeeds.
export function request_options_with_mediation_required(manifest_filename) {
  if (manifest_filename === undefined) {
    manifest_filename = "manifest.py";
  }
  const manifest_path = `${manifest_origin}/\
credential-management/support/fedcm/${manifest_filename}`;
  return {
    identity: {
      providers: [{
        configURL: manifest_path,
        clientId: '1',
        nonce: '2'
      }]
    },
    mediation: 'required'
  };
}

// Returns alternate FedCM CredentialRequestOptions for which navigator.credentials.get()
// succeeds.
export function alt_request_options_with_mediation_required(manifest_filename) {
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
        nonce: '2'
      }]
    },
    mediation: 'required'
  };
}

// Returns FedCM CredentialRequestOptions with auto re-authentication.
// succeeds.
export function request_options_with_mediation_optional(manifest_filename) {
  let options = alt_request_options_with_mediation_required(manifest_filename);
  // Approved client
  options.identity.providers[0].clientId = '123';
  options.mediation = 'optional';

  return options;
}

export function request_options_with_context(manifest_filename, context) {
  if (manifest_filename === undefined) {
    manifest_filename = "manifest.py";
  }
  const manifest_path = `${manifest_origin}/\
credential-management/support/fedcm/${manifest_filename}`;
  return {
    identity: {
      providers: [{
        configURL: manifest_path,
        clientId: '1',
        nonce: '2'
      }],
      context: context
    },
    mediation: 'required'
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
      ? '' : `?manifest_url=${manifest_url}`;

  return new Promise(resolve => {
    const img = document.createElement('img');
    img.src = `/credential-management/support/fedcm/select_manifest_in_root_manifest.py${url_query}`;
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

export function request_options_with_login_hint(manifest_filename, login_hint) {
  let options = request_options_with_mediation_required(manifest_filename);
  options.identity.providers[0].loginHint = login_hint;

  return options;
}

export function request_options_with_domain_hint(manifest_filename, domain_hint) {
  let options = request_options_with_mediation_required(manifest_filename);
  options.identity.providers[0].domainHint = domain_hint;

  return options;
}

export function fedcm_get_dialog_type_promise(t) {
  return new Promise(resolve => {
    async function helper() {
      // Try to get the dialog type. If the UI is not up yet, we'll catch an exception
      // and try again in 100ms.
      try {
        const type = await window.test_driver.get_fedcm_dialog_type();
        resolve(type);
      } catch (ex) {
        t.step_timeout(helper, 100);
      }
    }
    helper();
  });
}

export function fedcm_get_title_promise(t) {
  return new Promise(resolve => {
    async function helper() {
      // Try to get the title. If the UI is not up yet, we'll catch an exception
      // and try again in 100ms.
      try {
        const title = await window.test_driver.get_fedcm_dialog_title();
        resolve(title);
      } catch (ex) {
        t.step_timeout(helper, 100);
      }
    }
    helper();
  });
}

export function fedcm_select_account_promise(t, account_index) {
  return new Promise(resolve => {
    async function helper() {
      // Try to select the account. If the UI is not up yet, we'll catch an exception
      // and try again in 100ms.
      try {
        await window.test_driver.select_fedcm_account(account_index);
        resolve();
      } catch (ex) {
        t.step_timeout(helper, 100);
      }
    }
    helper();
  });
}

export function fedcm_get_and_select_first_account(t, options) {
  const credentialPromise = navigator.credentials.get(options);
  fedcm_select_account_promise(t, 0);
  return credentialPromise;
}

export function disconnect_options(accountHint, manifest_filename) {
  if (manifest_filename === undefined) {
    manifest_filename = "manifest.py";
  }
  const manifest_path = `${manifest_origin}/\
credential-management/support/fedcm/${manifest_filename}`;
  return {
      configURL: manifest_path,
      clientId: '1',
      accountHint: accountHint
      };
}

export function alt_disconnect_options(accountHint, manifest_filename) {
  if (manifest_filename === undefined) {
    manifest_filename = "manifest.py";
  }
  const manifest_path = `${alt_manifest_origin}/\
credential-management/support/fedcm/${manifest_filename}`;
  return {
      configURL: manifest_path,
      clientId: '1',
      accountHint: accountHint
  };
}
