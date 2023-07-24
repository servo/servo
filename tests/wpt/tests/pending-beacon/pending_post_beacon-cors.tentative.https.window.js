// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=./resources/pending_beacon-helper.js

'use strict';

const {HTTPS_ORIGIN, HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();
const SMALL_SIZE = 500;

for (const dataType in BeaconDataType) {
  postBeaconSendDataTest(
      dataType, generatePayload(SMALL_SIZE),
      `PendingPostBeacon[${dataType}]: same-origin`,
      {urlOptions: {host: HTTPS_ORIGIN, expectOrigin: HTTPS_ORIGIN}});

  postBeaconSendDataTest(
      dataType, generatePayload(SMALL_SIZE),
      `PendingPostBeacon[${dataType}]: cross-origin, ` +
          `CORS-safelisted Content-Type`,
      {
        urlOptions: {
          host: HTTPS_NOTSAMESITE_ORIGIN,
          expectOrigin: HTTPS_ORIGIN,
        }
      });

  postBeaconSendDataTest(
      dataType, generatePayload(SMALL_SIZE),
      `PendingPostBeacon[${dataType}]: cross-origin, ` +
          'CORS-safelisted Content-Type => ' +
          'non-CORS response (from redirect handler) ' +
          'should be rejected by browser',
      {
        expectCount: 0,
        urlOptions: {
          useRedirectHandler: true,
          host: HTTPS_NOTSAMESITE_ORIGIN,
        }
      });

  postBeaconSendDataTest(
      dataType, generatePayload(SMALL_SIZE),
      `PendingPostBeacon[${dataType}]: cross-origin, ` +
          'CORS-safelisted Content-Type => no cookie expected',
      {
        setCookie: 'test_beacon_cookie',
        urlOptions: {
          host: HTTPS_NOTSAMESITE_ORIGIN,
          expectOrigin: HTTPS_ORIGIN,
          expectCredentials: false,
        }
      });
}

postBeaconSendDataTest(
    BeaconDataType.Blob, generatePayload(SMALL_SIZE),
    'PendingPostBeacon[Blob]: cross-origin, non-CORS-safelisted Content-Type' +
        ' => preflight expected',
    {
      urlOptions: {
        host: HTTPS_NOTSAMESITE_ORIGIN,
        contentType: 'application/octet-stream',
        expectOrigin: HTTPS_ORIGIN,
        expectPreflight: true,
      }
    });
