// @ts-check
// Import the types from the TypeScript file
/**
 * @typedef {import('../dc-types').GetProtocol} GetProtocol
 * @typedef {import('../dc-types').DigitalCredentialGetRequest} DigitalCredentialGetRequest
 * @typedef {import('../dc-types').CredentialRequestOptions} CredentialRequestOptions
 * @typedef {import('../dc-types').CreateProtocol} CreateProtocol
 * @typedef {import('../dc-types').DigitalCredentialCreateRequest} DigitalCredentialCreateRequest
 * @typedef {import('../dc-types').CredentialCreationOptions} CredentialCreationOptions
 * @typedef {import('../dc-types').SendMessageData} SendMessageData
 * @typedef {import('../dc-types').MakeGetOptionsConfig} MakeGetOptionsConfig
 * @typedef {import('../dc-types').MakeCreateOptionsConfig} MakeCreateOptionsConfig
 * @typedef {import('../dc-types').CredentialMediationRequirement} CredentialMediationRequirement
 * @typedef {import('../dc-types').MobileDocumentRequest} MobileDocumentRequest
 * @typedef {GetProtocol | CreateProtocol} Protocol
 */

/** @type {Record<Protocol, object | MobileDocumentRequest>} */
const CANONICAL_REQUEST_OBJECTS = {
  openid4vci: {
    /* Canonical object coming soon */
  },
  "openid4vp-v1-unsigned": {
    "dcql_query": {
      "credentials": [
        {
          "id": "pid",
          "format": "dc+sd-jwt",
          "meta": {
            "vct_values": [
              "urn:eudi:pid:1"
            ]
          },
          "claims": [
            {
              "path": [
                "family_name"
              ]
            },
            {
              "path": [
                "given_name"
              ]
            }
          ]
        }
      ]
    },
    "nonce": "GGGRdhww2TFQM4dw6wgGc0suML49-._~",
    "client_metadata": {
      "vp_formats_supported": {
        "dc+sd-jwt": {
          "sd-jwt_alg_values": [
            "RS256",
            "RS384",
            "RS512",
            "PS256",
            "PS384",
            "PS512",
            "ES256",
            "ES256K",
            "ES384",
            "ES512",
            "EdDSA",
            "Ed25519",
            "Ed448"
          ],
          "kb-jwt_alg_values": [
            "RS256",
            "RS384",
            "RS512",
            "PS256",
            "PS384",
            "PS512",
            "ES256",
            "ES256K",
            "ES384",
            "ES512",
            "EdDSA",
            "Ed25519",
            "Ed448"
          ]
        }
      },
      "jwks": {
        "keys": [
          {
            "kty": "EC",
            "use": "enc",
            "crv": "P-256",
            "kid": "A541J5yUqazgE8WBFkIyeh2OtK-udqUR_OC0kB7l3oU",
            "x": "cwYyuS94hcOtcPlrMMtGtflCfbZUwz5Mf1Gfa2m0AM8",
            "y": "KB7sJkFQyB8jZHO9vmWS5LNECL4id3OJO9HX9ChNonA",
            "alg": "ECDH-ES"
          }
        ]
      },
      "encrypted_response_enc_values_supported": [
        "A128GCM"
      ]
    },
    "response_type": "vp_token",
    "response_mode": "dc_api.jwt",
    "O4Li4bl8jqjimxUt": "TarZDlUChNKhLCkv"
  },
  "openid4vp-v1-signed": {
    "request": "eyJ4NWMiOlsiTUlJSVZUQ0NCL3VnQXdJQkFnSVVHZmxJbEZ5dFk1clQ1aUI2RGRIMm9ZSS85L3N3Q2dZSUtvWkl6ajBFQXdJd0lURUxNQWtHQTFVRUJoTUNSMEl4RWpBUUJnTlZCQU1NQ1U5SlJFWWdWR1Z6ZERBZUZ3MHlOREV4TWpjeU1EUXdORGhhRncwek5ERXhNalV5TURRd05EaGFNQ0V4Q3pBSkJnTlZCQVlUQWtkQ01SSXdFQVlEVlFRRERBbFBTVVJHSUZSbGMzUXdXVEFUQmdjcWhrak9QUUlCQmdncWhrak9QUU1CQndOQ0FBVFQvZExzZDUxTExCckdWNlIyM282dnltUnhIWGVGQm9JOHlxMzF5NWtGVjJWVjBnaTl4NVp6RUZpcThETWlBSHVjTEFDRm5keEx0Wm9yQ2hhOXp6blFvNElIRHpDQ0J3c3dIUVlEVlIwT0JCWUVGTGx4dDJBQjR3R0xuREdsdW5BaElhRktFQllSTUI4R0ExVWRJd1FZTUJhQUZMbHh0MkFCNHdHTG5ER2x1bkFoSWFGS0VCWVJNQThHQTFVZEV3RUIvd1FGTUFNQkFmOHdnZ2EyQmdOVkhSRUVnZ2F0TUlJR3FZSVFkM2QzTG1obFpXNWhiaTV0WlM1MWE0SUpiRzlqWVd4b2IzTjBnaFpzYjJOaGJHaHZjM1F1WlcxdlltbDRMbU52TG5WcmdoMWtaVzF2TG1ObGNuUnBabWxqWVhScGIyNHViM0JsYm1sa0xtNWxkSUljZDNkM0xtTmxjblJwWm1sallYUnBiMjR1YjNCbGJtbGtMbTVsZElJZ2MzUmhaMmx1Wnk1alpYSjBhV1pwWTJGMGFXOXVMbTl3Wlc1cFpDNXVaWFNDSW1SbGJXOHVjR2xrTFdsemMzVmxjaTVpZFc1a1pYTmtjblZqYTJWeVpXa3VaR1dDTUhKbGRtbGxkeTFoY0hBdFpHVjJMV0p5WVc1amFDMHhMbU5sY25ScFptbGpZWFJwYjI0dWIzQmxibWxrTG01bGRJSXdjbVYyYVdWM0xXRndjQzFrWlhZdFluSmhibU5vTFRJdVkyVnlkR2xtYVdOaGRHbHZiaTV2Y0dWdWFXUXVibVYwZ2pCeVpYWnBaWGN0WVhCd0xXUmxkaTFpY21GdVkyZ3RNeTVqWlhKMGFXWnBZMkYwYVc5dUxtOXdaVzVwWkM1dVpYU0NNSEpsZG1sbGR5MWhjSEF0WkdWMkxXSnlZVzVqYUMwMExtTmxjblJwWm1sallYUnBiMjR1YjNCbGJtbGtMbTVsZElJd2NtVjJhV1YzTFdGd2NDMWtaWFl0WW5KaGJtTm9MVFV1WTJWeWRHbG1hV05oZEdsdmJpNXZjR1Z1YVdRdWJtVjBnakJ5WlhacFpYY3RZWEJ3TFdSbGRpMWljbUZ1WTJndE5pNWpaWEowYVdacFkyRjBhVzl1TG05d1pXNXBaQzV1WlhTQ01ISmxkbWxsZHkxaGNIQXRaR1YyTFdKeVlXNWphQzAzTG1ObGNuUnBabWxqWVhScGIyNHViM0JsYm1sa0xtNWxkSUl3Y21WMmFXVjNMV0Z3Y0Mxa1pYWXRZbkpoYm1Ob0xUZ3VZMlZ5ZEdsbWFXTmhkR2x2Ymk1dmNHVnVhV1F1Ym1WMGdqQnlaWFpwWlhjdFlYQndMV1JsZGkxaWNtRnVZMmd0T1M1alpYSjBhV1pwWTJGMGFXOXVMbTl3Wlc1cFpDNXVaWFNDTVhKbGRtbGxkeTFoY0hBdFpHVjJMV0p5WVc1amFDMHhNQzVqWlhKMGFXWnBZMkYwYVc5dUxtOXdaVzVwWkM1dVpYU0NNWEpsZG1sbGR5MWhjSEF0WkdWMkxXSnlZVzVqYUMweE1TNWpaWEowYVdacFkyRjBhVzl1TG05d1pXNXBaQzV1WlhTQ01YSmxkbWxsZHkxaGNIQXRaR1YyTFdKeVlXNWphQzB4TWk1alpYSjBhV1pwWTJGMGFXOXVMbTl3Wlc1cFpDNXVaWFNDTVhKbGRtbGxkeTFoY0hBdFpHVjJMV0p5WVc1amFDMHhNeTVqWlhKMGFXWnBZMkYwYVc5dUxtOXdaVzVwWkM1dVpYU0NNWEpsZG1sbGR5MWhjSEF0WkdWMkxXSnlZVzVqYUMweE5DNWpaWEowYVdacFkyRjBhVzl1TG05d1pXNXBaQzV1WlhTQ01YSmxkbWxsZHkxaGNIQXRaR1YyTFdKeVlXNWphQzB4TlM1alpYSjBhV1pwWTJGMGFXOXVMbTl3Wlc1cFpDNXVaWFNDTVhKbGRtbGxkeTFoY0hBdFpHVjJMV0p5WVc1amFDMHhOaTVqWlhKMGFXWnBZMkYwYVc5dUxtOXdaVzVwWkM1dVpYU0NNWEpsZG1sbGR5MWhjSEF0WkdWMkxXSnlZVzVqYUMweE55NWpaWEowYVdacFkyRjBhVzl1TG05d1pXNXBaQzV1WlhTQ01YSmxkbWxsZHkxaGNIQXRaR1YyTFdKeVlXNWphQzB4T0M1alpYSjBhV1pwWTJGMGFXOXVMbTl3Wlc1cFpDNXVaWFNDTVhKbGRtbGxkeTFoY0hBdFpHVjJMV0p5WVc1amFDMHhPUzVqWlhKMGFXWnBZMkYwYVc5dUxtOXdaVzVwWkM1dVpYU0NNWEpsZG1sbGR5MWhjSEF0WkdWMkxXSnlZVzVqYUMweU1DNWpaWEowYVdacFkyRjBhVzl1TG05d1pXNXBaQzV1WlhTQ01YSmxkbWxsZHkxaGNIQXRaR1YyTFdKeVlXNWphQzB5TVM1alpYSjBhV1pwWTJGMGFXOXVMbTl3Wlc1cFpDNXVaWFNDTVhKbGRtbGxkeTFoY0hBdFpHVjJMV0p5WVc1amFDMHlNaTVqWlhKMGFXWnBZMkYwYVc5dUxtOXdaVzVwWkM1dVpYU0NNWEpsZG1sbGR5MWhjSEF0WkdWMkxXSnlZVzVqYUMweU15NWpaWEowYVdacFkyRjBhVzl1TG05d1pXNXBaQzV1WlhTQ01YSmxkbWxsZHkxaGNIQXRaR1YyTFdKeVlXNWphQzB5TkM1alpYSjBhV1pwWTJGMGFXOXVMbTl3Wlc1cFpDNXVaWFNDTVhKbGRtbGxkeTFoY0hBdFpHVjJMV0p5WVc1amFDMHlOUzVqWlhKMGFXWnBZMkYwYVc5dUxtOXdaVzVwWkM1dVpYU0NNWEpsZG1sbGR5MWhjSEF0WkdWMkxXSnlZVzVqYUMweU5pNWpaWEowYVdacFkyRjBhVzl1TG05d1pXNXBaQzV1WlhTQ01YSmxkbWxsZHkxaGNIQXRaR1YyTFdKeVlXNWphQzB5Tnk1alpYSjBhV1pwWTJGMGFXOXVMbTl3Wlc1cFpDNXVaWFNDTVhKbGRtbGxkeTFoY0hBdFpHVjJMV0p5WVc1amFDMHlPQzVqWlhKMGFXWnBZMkYwYVc5dUxtOXdaVzVwWkM1dVpYU0NNWEpsZG1sbGR5MWhjSEF0WkdWMkxXSnlZVzVqYUMweU9TNWpaWEowYVdacFkyRjBhVzl1TG05d1pXNXBaQzV1WlhTQ01YSmxkbWxsZHkxaGNIQXRaR1YyTFdKeVlXNWphQzB6TUM1alpYSjBhV1pwWTJGMGFXOXVMbTl3Wlc1cFpDNXVaWFF3Q2dZSUtvWkl6ajBFQXdJRFNBQXdSUUlnRGhrekYrS1hWdWFvNVo5bFUycU1TY21rZ3JQUTVNQnRVUFZkcXRUdFpwd0NJUURNVWw1b2ZqcDEvNG1OWHorZ3BTejVvcW1oVzloUzRJaFJoQXMvQWxSNDB3PT0iXSwia2lkIjoiNUgxV0xlU3g1NXRNVzZKTmx2cU1mZzNPX0UwZVFQcUI4akRTb1VuNm9pSSIsInR5cCI6Im9hdXRoLWF1dGh6LXJlcStqd3QiLCJhbGciOiJFUzI1NiJ9.eyJhdWQiOiJodHRwczovL3NlbGYtaXNzdWVkLm1lL3YyIiwiclVMYXNsejVQczhuUW5qbiI6IjJFNjJSb0g0Z09Ua2RCNTEiLCJyZXNwb25zZV90eXBlIjoidnBfdG9rZW4iLCJleHBlY3RlZF9vcmlnaW5zIjpbImh0dHBzOi8vZGVtby5jZXJ0aWZpY2F0aW9uLm9wZW5pZC5uZXQiXSwiZGNxbF9xdWVyeSI6eyJjcmVkZW50aWFscyI6W3siaWQiOiJtZGwiLCJmb3JtYXQiOiJtc29fbWRvYyIsIm1ldGEiOnsiZG9jdHlwZV92YWx1ZSI6Im9yZy5pc28uMTgwMTMuNS4xLm1ETCJ9LCJjbGFpbXMiOlt7InBhdGgiOlsib3JnLmlzby4xODAxMy41LjEiLCJmYW1pbHlfbmFtZSJdfSx7InBhdGgiOlsib3JnLmlzby4xODAxMy41LjEiLCJnaXZlbl9uYW1lIl19XX1dfSwibm9uY2UiOiJhdnVyOExxNFFTWXFCS1ZVVWpScUhQY3haTDhmLS5ffiIsImNsaWVudF9pZCI6Ing1MDlfaGFzaDpOMklqYlM5cS1lM0JWTUViZHEyNTJicFVIcUpmeVExdWRUVzNNTk9sLUU0IiwiY2xpZW50X21ldGFkYXRhIjp7InZwX2Zvcm1hdHNfc3VwcG9ydGVkIjp7Im1zb19tZG9jIjp7ImFsZyI6WyJFUzI1NiJdfX0sImp3a3MiOnsia2V5cyI6W3sia3R5IjoiRUMiLCJ1c2UiOiJlbmMiLCJjcnYiOiJQLTI1NiIsImtpZCI6IkE1NDFKNXlVcWF6Z0U4V0JGa0l5ZWgyT3RLLXVkcVVSX09DMGtCN2wzb1UiLCJ4IjoiY3dZeXVTOTRoY090Y1Bsck1NdEd0ZmxDZmJaVXd6NU1mMUdmYTJtMEFNOCIsInkiOiJLQjdzSmtGUXlCOGpaSE85dm1XUzVMTkVDTDRpZDNPSk85SFg5Q2hOb25BIiwiYWxnIjoiRUNESC1FUyJ9XX0sImVuY3J5cHRlZF9yZXNwb25zZV9lbmNfdmFsdWVzX3N1cHBvcnRlZCI6WyJBMTI4R0NNIl19LCJyZXNwb25zZV9tb2RlIjoiZGNfYXBpLmp3dCJ9.-LExZpyxOuwtwu2l7fiDJL1mf5ZLTBEkVfXFQeLvp9A95qwUYqEsJ81vZEtztnNZHKhLwL8AeQsPDujW3EQUFg"
  },
  "openid4vp-v1-multisigned": {
    "signatures": [
      {
        "signature": "__lsLh09iG2M569VWsm7r89pycGuhefGa66bbj7baitszgWYQkY_qELtqjdxioZ2sR2VulIHphc7qm9IhOOUpg",
        "protected": "eyJhbGciOiJFUzI1NiIsImtpZCI6ImRlbW8ta2V5LTEtZXMyNTYifQ"
      },
      {
        "signature": "Ra1GZSjOvJXDpO15Yy6hZzv-ZT-aLtCPdBt7_QNw2L-bDMRp64qgPzjDco4-w4DqvrTBWczsO9Za7gR768ks5G4ZzhnzCHnTyp3CqZJIo1ZpPNtVk6Pkk2xN5AqQx9idDzPmc57BCP6bE02iOcnTh4KUNfuZwW2Dmw5WWtMDr1aCpX_AEGsdc5rn7lSKCAtyF_mjrgLdHDS-i2jtnJGPxEtr586e9JvApQc_D-a1KKHN4iubNmerAZwhUks-ATM7rEnvvSGy9mDX9dQdAg_3Lrkha4B3Rtpmn8cq45mBII4xxt49rwi8VgWgd1gRBgwyQuycsiXpZwaWKmIxA7sqPw",
        "protected": "eyJhbGciOiJSUzI1NiIsImtpZCI6ImRlbW8ta2V5LTItcnMyNTYifQ"
      },
      {
        "signature": "h_ro8GMqMeq5C2e0oJqktenk1Yc5uO3plZxq-a4dl45jT_1jTONRpRmqrl_YakKtkPFoGfx7oavdi1_qdTcUCw",
        "protected": "eyJhbGciOiJFZERTQSIsImtpZCI6ImRlbW8ta2V5LTMtZWRkc2EifQ"
      }
    ],
    "payload": "eyJkY3FsX3F1ZXJ5Ijp7ImNyZWRlbnRpYWxzIjpbeyJpZCI6InBpZCIsImZvcm1hdCI6ImRjK3NkLWp3dCIsIm1ldGEiOnsidmN0X3ZhbHVlcyI6WyJ1cm46ZXVkaTpwaWQ6MSJdfSwiY2xhaW1zIjpbeyJwYXRoIjpbImZhbWlseV9uYW1lIl19LHsicGF0aCI6WyJnaXZlbl9uYW1lIl19XX1dfSwibm9uY2UiOiJHR0dSZGh3dzJURlFNNGR3NndnR2Mwc3VNTDQ5LS5ffiIsImNsaWVudF9tZXRhZGF0YSI6eyJ2cF9mb3JtYXRzX3N1cHBvcnRlZCI6eyJkYytzZC1qd3QiOnsic2Qtand0X2FsZ192YWx1ZXMiOlsiUlMyNTYiLCJSUzM4NCIsIlJTNTEyIiwiUFMyNTYiLCJQUzM4NCIsIlBTNTEyIiwiRVMyNTYiLCJFUzI1NksiLCJFUzM4NCIsIkVTNTEyIiwiRWREU0EiLCJFZDI1NTE5IiwiRWQ0NDgiXSwia2Itand0X2FsZ192YWx1ZXMiOlsiUlMyNTYiLCJSUzM4NCIsIlJTNTEyIiwiUFMyNTYiLCJQUzM4NCIsIlBTNTEyIiwiRVMyNTYiLCJFUzI1NksiLCJFUzM4NCIsIkVTNTEyIiwiRWREU0EiLCJFZDI1NTE5IiwiRWQ0NDgiXX19LCJqd2tzIjp7ImtleXMiOlt7Imt0eSI6IkVDIiwidXNlIjoiZW5jIiwiY3J2IjoiUC0yNTYiLCJraWQiOiJBNTQxSjV5VXFhemdFOFdCRmtJeWVoMk90Sy11ZHFVUl9PQzBrQjdsM29VIiwieCI6ImN3WXl1Uzk0aGNPdGNQbHJNTXRHdGZsQ2ZiWlV3ejVNZjFHZmEybTBBTTgiLCJ5IjoiS0I3c0prRlF5QjhqWkhPOXZtV1M1TE5FQ0w0aWQzT0pPOUhYOUNoTm9uQSIsImFsZyI6IkVDREgtRVMifV19LCJlbmNyeXB0ZWRfcmVzcG9uc2VfZW5jX3ZhbHVlc19zdXBwb3J0ZWQiOlsiQTEyOEdDTSJdfSwicmVzcG9uc2VfdHlwZSI6InZwX3Rva2VuIiwicmVzcG9uc2VfbW9kZSI6ImRjX2FwaS5qd3QiLCJPNExpNGJsOGpxamlteFV0IjoiVGFyWkRsVUNoTktoTENrdiJ9"
  },
  /** @type MobileDocumentRequest **/
  "org-iso-mdoc": {
    deviceRequest:
      "omd2ZXJzaW9uYzEuMGtkb2NSZXF1ZXN0c4GhbGl0ZW1zUmVxdWVzdNgYWIKiZ2RvY1R5cGV1b3JnLmlzby4xODAxMy41LjEubURMam5hbWVTcGFjZXOhcW9yZy5pc28uMTgwMTMuNS4x9pWthZ2Vfb3Zlcl8yMfRqZ2l2ZW5fbmFtZfRrZmFtaWx5X25hbWX0cmRyaXZpbmdfcHJpdmlsZWdlc_RocG9ydHJhaXT0",
    encryptionInfo:
      "gmVkY2FwaaJlbm9uY2VYICBetSsDkKlE_G9JSIHwPzr3ctt6Ol9GgmCH8iGdGQNJcnJlY2lwaWVudFB1YmxpY0tleaQBAiABIVggKKm1iPeuOb9bDJeeJEL4QldYlWvY7F_K8eZkmYdS9PwiWCCm9PLEmosiE_ildsE11lqq4kDkjhfQUKPpbX-Hm1ZSLg",
  },
};

/**
 * Internal helper to create final options from a list of requests.
 *
 * @template {DigitalCredentialGetRequest[] | DigitalCredentialCreateRequest[]} TRequests
 * @template {CredentialRequestOptions | CredentialCreationOptions} TOptions
 * @param {TRequests} requests
 * @param {CredentialMediationRequirement} [mediation]
 * @param {AbortSignal} [signal]
 * @returns {TOptions}
 */
function makeOptionsFromRequests(requests, mediation, signal) {
  /** @type {TOptions} */
  const options = /** @type {TOptions} */ ({ digital: { requests } });

  if (mediation) {
    options.mediation = mediation;
  }

  if (signal) {
    options.signal = signal;
  }

  return options;
}

/**
 * Build requests from protocols, using canonical data for each protocol.
 * For create operations with explicit data, uses that data for all protocols.
 *
 * @template Req
 * @param {Protocol[]} protocols
 * @param {Record<string, (data?: MobileDocumentRequest | object) => Req>} mapping
 * @param {MobileDocumentRequest | object} [explicitData] - Explicit data for create operations
 * @returns {Req[]}
 * @throws {Error} If an unknown protocol string is encountered.
 */
function buildRequestsFromProtocols(protocols, mapping, explicitData) {
  return protocols.map((protocol) => {
    if (!(protocol in mapping)) {
      throw new Error(`Unknown request type within array: ${protocol}`);
    }
    // Use explicit data if provided (for create with data), otherwise canonical data
    return mapping[protocol](explicitData);
  });
}

/** @type {{
 *   get: Record<GetProtocol, (data?: MobileDocumentRequest | object) => DigitalCredentialGetRequest>;
 *   create: Record<CreateProtocol, (data?: object) => DigitalCredentialCreateRequest>;
 * }} */
const allMappings = {
  get: {
    "org-iso-mdoc": (
      data = { ...CANONICAL_REQUEST_OBJECTS["org-iso-mdoc"] },
    ) => {
      return { protocol: "org-iso-mdoc", data };
    },
    "openid4vp-v1-unsigned": (
      data = { ...CANONICAL_REQUEST_OBJECTS["openid4vp-v1-unsigned"] },
    ) => {
      return { protocol: "openid4vp-v1-unsigned", data };
    },
    "openid4vp-v1-signed": (
      data = { ...CANONICAL_REQUEST_OBJECTS["openid4vp-v1-signed"] },
    ) => {
      return { protocol: "openid4vp-v1-signed", data };
    },
    "openid4vp-v1-multisigned": (
      data = { ...CANONICAL_REQUEST_OBJECTS["openid4vp-v1-multisigned"] },
    ) => {
      return { protocol: "openid4vp-v1-multisigned", data };
    },
  },
  create: {
    "openid4vci": (data = { ...CANONICAL_REQUEST_OBJECTS["openid4vci"] }) => {
      return { protocol: "openid4vci", data };
    },
  },
};

/**
 * Generic helper to create credential options from config with protocol already set.
 * @template {MakeGetOptionsConfig | MakeCreateOptionsConfig} TConfig
 * @template {DigitalCredentialGetRequest | DigitalCredentialCreateRequest} TRequest
 * @template {CredentialRequestOptions | CredentialCreationOptions} TOptions
 * @param {TConfig} config - Configuration options with protocol already defaulted
 * @param {Record<string, (data?: MobileDocumentRequest | object) => TRequest>} mapping - Protocol to request mapping
 * @returns {TOptions}
 */
function makeCredentialOptionsFromConfig(config, mapping) {
  const { protocol, requests = [], data, mediation, signal } = config;

  // Validate that we have either a protocol or requests
  if (!protocol && !requests?.length) {
    throw new Error("No protocol. Can't make options.");
  }

  /** @type {TRequest[]} */
  const allRequests = [];

  allRequests.push(.../** @type {TRequest[]} */ (requests));

  if (protocol) {
    const protocolArray = Array.isArray(protocol) ? protocol : [protocol];
    const protocolRequests = buildRequestsFromProtocols(protocolArray, mapping, data);
    allRequests.push(...protocolRequests);
  }

  return /** @type {TOptions} */ (makeOptionsFromRequests(allRequests, mediation, signal));
}

/**
 * Creates options for getting credentials.
 * @export
 * @param {MakeGetOptionsConfig} [config={}] - Configuration options
 * @returns {CredentialRequestOptions}
 */
export function makeGetOptions(config = {}) {
  /** @type {MakeGetOptionsConfig} */
  const configWithDefaults = {
    protocol: ["openid4vp-v1-unsigned", "org-iso-mdoc"],
    ...config,
  };

  return /** @type {CredentialRequestOptions} */ (
    makeCredentialOptionsFromConfig(configWithDefaults, allMappings.get)
  );
}

/**
 * Creates options for creating credentials.
 * @export
 * @param {MakeCreateOptionsConfig} [config={}] - Configuration options
 * @returns {CredentialCreationOptions}
 */
export function makeCreateOptions(config = {}) {
  /** @type {MakeCreateOptionsConfig} */
  const configWithDefaults = {
    protocol: "openid4vci",
    ...config,
  };

  return /** @type {CredentialCreationOptions} */ (
    makeCredentialOptionsFromConfig(configWithDefaults, allMappings.create)
  );
}

/**
 * Sends a message to an iframe and return the response.
 *
 * @param {HTMLIFrameElement} iframe - The iframe element to send the message to.
 * @param {SendMessageData} data - The data to be sent to the iframe.
 * @returns {Promise<any>} - A promise that resolves with the response from the iframe.
 */
export function sendMessage(iframe, data) {
  return new Promise((resolve, reject) => {
    if (!iframe.contentWindow) {
      reject(
        new Error(
          "iframe.contentWindow is undefined, cannot send message (something is wrong with the test that called this).",
        ),
      );
      return;
    }
    window.addEventListener("message", function messageListener(event) {
      if (event.source === iframe.contentWindow) {
        window.removeEventListener("message", messageListener);
        resolve(event.data);
      }
    });
    iframe.contentWindow.postMessage(data, "*");
  });
}

/**
 * Load an iframe with the specified URL and wait for it to load.
 *
 * @param {HTMLIFrameElement} iframe
 * @param {string|URL} url
 * @returns {Promise<void>}
 */
export function loadIframe(iframe, url) {
  return new Promise((resolve, reject) => {
    iframe.addEventListener("load", () => resolve(), { once: true });
    iframe.addEventListener("error", (event) => reject(event.error), {
      once: true,
    });
    if (!iframe.isConnected) {
      document.body.appendChild(iframe);
    }
    iframe.src = url.toString();
  });
}
