// Builds valid digital identity request for navigator.identity.get() API.
export function buildValidNavigatorIdentityRequest() {
  return {
      digital: {
        providers: [{
          protocol: "urn:openid.net:oid4vp",
          request: JSON.stringify({
            // Based on https://github.com/openid/OpenID4VP/issues/125
            client_id: "client.example.org",
            client_id_scheme: "web-origin",
            nonce: "n-0S6_WzA2Mj",
            presentation_definition: {
              // Presentation Exchange request, omitted for brevity
            }
          }),
        }],
      },
  };
}

// Requests digital identity with user activation.
export function requestIdentityWithActivation(test_driver, request) {
  return test_driver.bless("request identity with activation", async function() {
    return await navigator.identity.get(request);
  });
}
