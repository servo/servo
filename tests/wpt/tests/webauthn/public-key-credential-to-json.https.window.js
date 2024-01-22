// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/resources/utils.js
// META: script=helpers.js

function assertObjectKeysEq(a, b) {
  let a_keys = new Set(Object.keys(a));
  let b_keys = new Set(Object.keys(b));
  assert_true(
      a_keys.length == b_keys.length && [...a_keys].every(k => b_keys.has(k)),
      `keys differ: ${a_keys} != ${b_keys}`);
}

// Returns the JSON encoding for `value`. If `value` is a function, `optParent`
// is the object to which execution should be bound.
function convertValue(value, optParent) {
  switch (typeof value) {
    case 'undefined':
    case 'boolean':
    case 'number':
    case 'bigint':
    case 'string':
    case 'symbol':
      return value;
    case 'function':
      return value.apply(optParent);
    case 'object':
      if (value.__proto__.constructor === Object) {
        var result = {};
        Object.entries(value).map((k, v) => {
          result[k] = convertValue(k, v);
        });
        return result;
      }
      if (value instanceof Array) {
        return value.map(convertValue);
      }
      if (value instanceof ArrayBuffer) {
        return base64urlEncode(new Uint8Array(value));
      }
      throw `can't convert value ${value} in ${parent}`;
    default:
      throw `${value} has unexpected type`;
  }
}

// Conversion spec for a single attribute.
// @typedef {Object} ConvertParam
// @property {string} name - The name of the attribute to convert from
// @property {string=} target - The name of the attribute to convert to, if
// different from `name`
// @property {function=} func - Method to convert this property. Defaults to
// convertValue().

// Returns the JSON object for `obj`.
//
// @param obj
// @param {Array<(string|ConvertParam)>} keys - The names of parameters in
// `obj` to convert, or instances of ConvertParam for complex cases.
function convertObject(obj, params) {
  let result = {};
  params.forEach((param) => {
    switch (typeof (param)) {
      case 'string':
        assert_true(param in obj, `missing ${param}`);
        if (obj[param] !== null) {
          result[param] = convertValue(obj[param], obj);
        }
        break;
      case 'object':
        assert_true(param.name in obj, `missing ${param.name}`);
        const val = obj[param.name];
        const target_key = param.target || param.name;
        const convert_func = param.func || convertValue;
        try {
          result[target_key] =
              convert_func(((typeof val) == 'function' ? val.apply(obj) : val));
        } catch (e) {
          throw `failed to convert ${param.name}: ${e}`
        }
        break;
      default:
        throw `invalid key ${param}`;
    }
  });
  return result;
}

// Converts an AuthenticatorResponse instance into a JSON object.
// @param {!AuthenticatorResponse}
function authenticatorResponseToJson(response) {
  assert_true(
      (response instanceof AuthenticatorAttestationResponse) ||
      (response instanceof AuthenticatorAssertionResponse));
  const isAttestation = (response instanceof AuthenticatorAttestationResponse);
  const keys =
      (isAttestation ?
           [
             'clientDataJSON', 'attestationObject',
             {name: 'getAuthenticatorData', target: 'authenticatorData'},
             {name: 'getPublicKey', target: 'publicKey'},
             {name: 'getPublicKeyAlgorithm', target: 'publicKeyAlgorithm'},
             {name: 'getTransports', target: 'transports'}
           ] :
           ['clientDataJSON', 'authenticatorData', 'signature', 'userHandle']);
  return convertObject(response, keys);
}

// Converts a PublicKeyCredential instance to a JSON object.
// @param {!PublicKeyCredential}
function publicKeyCredentialToJson(cred) {
  const keys = [
    'id', 'rawId', {name: 'response', func: authenticatorResponseToJson},
    'authenticatorAttachment',
    {name: 'getClientExtensionResults', target: 'clientExtensionResults'},
    'type'
  ];
  return convertObject(cred, keys);
}

virtualAuthenticatorPromiseTest(
    async t => {
      let credential = await createCredential();
      assertJsonEquals(
          credential.toJSON(), publicKeyCredentialToJson(credential));

      let assertion = await assertCredential(credential);
      assertJsonEquals(
          assertion.toJSON(), publicKeyCredentialToJson(assertion));
    },
    {
      protocol: 'ctap2_1',
      transport: 'usb',
    },
    'toJSON()');
