'use strict;'

let BA = {};

(function(BA) {
const TestPrivateKey = new Uint8Array([
  0xff, 0x1f, 0x47, 0xb1, 0x68, 0xb6, 0xb9, 0xea, 0x65, 0xf7, 0x97,
  0x4f, 0xf2, 0x2e, 0xf2, 0x36, 0x94, 0xe2, 0xf6, 0xb6, 0x8d, 0x66,
  0xf3, 0xa7, 0x64, 0x14, 0x28, 0xd4, 0x45, 0x35, 0x01, 0x8f
]);

const _hpkeModulePromise = import('../third_party/hpke-js/hpke.js');

// Common utilities.

function _get16(buffer, offset) {
  return buffer[offset] << 8 | buffer[offset + 1];
}

function _get32(buffer, offset) {
  return buffer[offset] << 24 | buffer[offset + 1] << 16 |
      buffer[offset + 2] << 8 | buffer[offset + 3];
}

function _put16(buffer, offset, val) {
  buffer[offset] = val >> 8;
  buffer[offset + 1] = val & 0xFF;
}

function _put32(buffer, offset, val) {
  buffer[offset] = (val >> 24) & 0xFF;
  buffer[offset + 1] = (val >> 16) & 0xFF;
  buffer[offset + 2] = (val >> 8) & 0xFF;
  buffer[offset + 3] = val & 0xFF;
}

// Concatenates two Uint8Array's.
function _concat(a, b) {
  let c = new Uint8Array(a.length + b.length);
  for (var i = 0; i < a.length; ++i) {
    c[i] = a[i];
  }
  for (var i = 0; i < b.length; ++i) {
    c[i + a.length] = b[i];
  }
  return c;
}

function _toArrayBuffer(typedArray) {
  return typedArray.buffer.slice(
      typedArray.byteOffset, typedArray.byteOffset + typedArray.byteLength);
}

function _toBytesArrayBuffer(str) {
  return _toArrayBuffer(new TextEncoder().encode(str));
}

function _bufferAsStream(buffer) {
  return new ReadableStream({
    start: controller => {
      controller.enqueue(buffer);
      controller.close();
    }
  });
}

// Returns an ArrayBuffer.
async function _applyTransform(inData, transform) {
  const resultResponse =
      new Response(_bufferAsStream(inData).pipeThrough(transform));
  const resultBlob = await resultResponse.blob();
  return await resultBlob.arrayBuffer();
}

// Returns an ArrayBuffer (promise).
async function _gzip(inData) {
  const compress = new CompressionStream('gzip');
  return _applyTransform(inData, compress);
}

// Returns an ArrayBuffer (promise).
async function _gunzip(inData) {
  const decompress = new DecompressionStream('gzip');
  return _applyTransform(inData, decompress);
}

// InterestGroupData decoding helpers.

function _decodeIgDataHeader(igData) {
  if (igData.length < 8) {
    throw 'Not enough data for B&A and OHTTP headers';
  }
  return {
    version: igData[0],
    keyId: igData[1],
    kemId: _get16(igData, 2),
    kdfId: _get16(igData, 4),
    aeadId: _get16(igData, 6),
    payload: igData.slice(8)
  };
}

// Splits up the actual B&A IG Data into the enc and ct portions
// for HPKE, using `suite` for sizing; and also figures out the appropriate
// info string.
function _splitIgDataPayloadIntoEncAndCt(header, suite) {
  const RequestMessageType = 'message/auction request';

  // From RFC 9458 (Oblivious HTTP):
  // "2.  Build a sequence of bytes (info) by concatenating the ASCII-
  //      encoded string "message/bhttp request"; a zero byte; key_id as an
  //      8-bit integer; plus kem_id, kdf_id, and aead_id as three 16-bit
  //      integers."
  // (except we use a different message type string).
  const infoLength = RequestMessageType.length + 1 + 1 + 6;
  let info = new Uint8Array(infoLength);
  for (let pos = 0; pos < RequestMessageType.length; ++pos) {
    info[pos] = RequestMessageType.charCodeAt(pos);
  }
  info[RequestMessageType.length] = 0;
  info[RequestMessageType.length + 1] = header.keyId;
  _put16(info, RequestMessageType.length + 2, header.kemId);
  _put16(info, RequestMessageType.length + 4, header.kdfId);
  _put16(info, RequestMessageType.length + 6, header.aeadId);
  return {
    info: info,
    enc: header.payload.slice(0, suite.kem.encSize),
    ct: header.payload.slice(suite.kem.encSize)
  };
}

// Unwraps the padding envelope.
function _decodeIgDataPaddingHeader(decryptedText) {
  let length = _get32(decryptedText, 1);
  let format = decryptedText[0];

  // We currently only support format 2, which version = 0, and gzip
  // compression.
  assert_equals(format, 2);
  return {
    format: format,
    data: decryptedText.slice(5, 5 + length)
  };
}

// serverResponse encoding helpers.

// Takes an ArrayBuffer, returns a Uint8Array.
function _frameServerResponse(arrayBuffer) {
  let array = new Uint8Array(arrayBuffer);
  let framedLength = 5 + array.length;
  let framed = new Uint8Array(framedLength);
  framed[0] = 2;  // gzip + ver 0.
  _put32(framed, 1, array.length);
  for (let i = 0; i < array.length; ++i) {
    framed[i + 5] = array[i];
  }
  return framed;
}

async function _encryptServerResponse(payload, decoded) {
  // This again follows RFC 9458 (Oblivious HTTP), "Encapsulation of
  // Responses", just with different message type:
  const ResponseMessageType = 'message/auction response';
  const Nk = decoded.cipherSuite.aead.keySize;
  const Nn = decoded.cipherSuite.aead.nonceSize;
  let secret = await decoded.receiveContext.export(
      _toBytesArrayBuffer(ResponseMessageType), Math.max(Nk, Nn));
  let responseNonce = new Uint8Array(Math.max(Nk, Nn));
  crypto.getRandomValues(responseNonce);
  let salt = _concat(decoded.enc, responseNonce);
  let prk = await decoded.cipherSuite.kdf.extract(salt, secret);
  let aeadKey =
      await decoded.cipherSuite.kdf.expand(prk, _toBytesArrayBuffer('key'), Nk);
  let aeadNonce = await decoded.cipherSuite.kdf.expand(
      prk, _toBytesArrayBuffer('nonce'), Nn);
  let encContext = decoded.cipherSuite.aead.createEncryptionContext(aeadKey);
  let ct = await encContext.seal(
      /*iv=*/ aeadNonce, /*data=*/ payload,
      /*aad=*/ _toBytesArrayBuffer(''));
  return _concat(responseNonce, new Uint8Array(ct));
}

// CBOR requires property names to be in sorted order; but the library we use
// doesn't do it automatically. Since it's easy for a test to fail for the
// wrong reason if the response isn't specified correctly, this ensures the
// proper ordering. It assumes a very simple data model, so no arrays with
// holes, no mixture of different kinds of indices in the map, etc.
// Getting the sort order right in more complicated cases is outside the
// scope of this helper.
function _sortForCbor(input) {
  if (input === null || typeof input !== 'object') {
    return input;
  }

  if (input instanceof Array) {
    let out = [];
    for (let i = 0; i < input.length; ++i) {
      out[i] = _sortForCbor(input[i]);
    }
    return out;
  } else if (input instanceof Uint8Array) {
    return input;
  } else {
    let keys = Object.getOwnPropertyNames(input).sort((a, b) => {
      // CBOR order compares lengths before values.
      if (a.length < b.length)
        return -1;
      if (a.length > b.length)
        return 1;
      if (a < b)
        return -1;
      if (a > b)
        return 1;
      return 0;
    });
    let out = {};
    for (let key of keys) {
      out[key] = _sortForCbor(input[key]);
    }
    return out;
  }
}

// Works on both ArrayBuffer and Uint8Array, returns the same type.
function _injectFault(input) {
  let uint8Input;
  if (input instanceof ArrayBuffer) {
    uint8Input = new Uint8Array(input);
  } else {
    assert_true(input instanceof Uint8Array);
    uint8Input = input;
  }

  // Just mess up the 0th byte.
  uint8Input[0] = uint8Input[0] ^ 0x4e;

  if (input instanceof ArrayBuffer) {
    return _toArrayBuffer(uint8Input);
  } else {
    return uint8Input;
  }
}

// Exported API.

// Decodes the request payload produced by getInterestGroupAdAuctionData into
// {paddedSize: ..., message: ..., cipherSuite: ... , receiveContext: ...,
//  enc:...}
BA.decodeInterestGroupData = async function(igData) {
  const hpke = await _hpkeModulePromise;

  // Decode B&A level headers, and check them.
  const header = _decodeIgDataHeader(igData);

  // Only version 0 in use now.
  assert_equals(header.version, 0);

  // Test config uses keyId = 0x14 only
  // If the feature is not set up properly we may get a different, non-test key.
  // We can't use assert_equals because it includes the (random) non-test key
  // in the error message if testing support for this feature is not present.
  assert_true(header.keyId === 0x14, "valid key Id");

  // Current cipher config.
  assert_equals(header.kemId, hpke.KemId.DhkemX25519HkdfSha256);
  assert_equals(header.kdfId, hpke.KdfId.HkdfSha256);
  assert_equals(header.aeadId, hpke.AeadId.Aes256Gcm);

  const suite = new hpke.CipherSuite({
    kem: header.kemId,
    kdf: header.kdfId,
    aead: header.aeadId,
  });

  // Split up the ciphertext from encapsulated key, and also compute
  // the expected message info.
  const pieces = _splitIgDataPayloadIntoEncAndCt(header, suite);

  // We can now decode the ciphertext.
  const privateKey = await suite.kem.importKey('raw', TestPrivateKey);
  const recipient = await suite.createRecipientContext(
      {recipientKey: privateKey, info: pieces.info, enc: pieces.enc});
  const pt = new Uint8Array(await recipient.open(pieces.ct));

  // The resulting text has yet another envelope with version and size info,
  // and a bunch of padding.
  const withoutPadding = _decodeIgDataPaddingHeader(pt);
  const decoded = CBOR.decode(_toArrayBuffer(withoutPadding.data));

  // Decompress IGs, CBOR-decode them, and replace in-place.
  for (let key of Object.getOwnPropertyNames(decoded.interestGroups)) {
    let val = decoded.interestGroups[key];
    let decompressedVal = await _gunzip(val);
    decoded.interestGroups[key] = CBOR.decode(decompressedVal);
  }

  return {
    paddedSize: pt.length,
    message: decoded,
    receiveContext: recipient,
    cipherSuite: suite,
    enc: pieces.enc
  };
};

BA.injectCborFault = 1;
BA.injectGzipFault = 2;
BA.injectFrameFault = 4;
BA.injectEncryptFault = 8;

// Encodes, compresses, encrypts, etc., `responseObject` into a proper
// serverResponse in reply to `decoded`.
BA.encodeServerResponse =
    async function(responseObject, decoded, injectFaults = 0) {
  let cborPayload = new Uint8Array(CBOR.encode(_sortForCbor(responseObject)));
  if (injectFaults & BA.injectCborFault) {
    cborPayload = _injectFault(cborPayload);
  }

  let gzipPayload = await _gzip(cborPayload);
  if (injectFaults & BA.injectGzipFault) {
    gzipPayload = _injectFault(gzipPayload);
  }

  let framedPayload = _toArrayBuffer(_frameServerResponse(gzipPayload));
  if (injectFaults & BA.injectFrameFault) {
    framedPayload = _injectFault(framedPayload);
  }

  let encrypted = await _encryptServerResponse(framedPayload, decoded);
  if (injectFaults & BA.injectEncryptFault) {
    encrypted = _injectFault(encrypted);
  }

  return encrypted;
};

// Returns a hash string that can be used to authorize a given response,
// formatted for use in an Ad-Auction-Result HTTP header.
BA.payloadHash = async function(serverResponse) {
  let hash =
      new Uint8Array(await crypto.subtle.digest('SHA-256', serverResponse));
  let hashString = ''
  for (let i = 0; i < hash.length; ++i) {
    hashString += String.fromCharCode(hash[i]);
  }
  return btoa(hashString)
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=+$/g, '');
};

// Authorizes each serverResponse hash in `hashes` to be used for
// B&A auction result.
BA.authorizeServerResponseHashes = async function(hashes) {
  let authorizeURL =
      new URL('resources/authorize-server-response.py', window.location);
  authorizeURL.searchParams.append('hashes', hashes.join(','));
  await fetch(authorizeURL, {adAuctionHeaders: true});
};

})(BA);
