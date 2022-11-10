'use strict';
const test_desc =
    'Calls to FUNCTION_NAME when device disconnects and discovery' +
    ' times out should reject promise rather than get stuck.';
let device;

bluetooth_test(
    async (t) => {
      let {device, fake_peripheral} =
          await getConnectedHealthThermometerDevice({
            filters: [{services: ['health_thermometer']}],
            optionalServices: ['generic_access']
          });

      await fake_peripheral.setNextGATTDiscoveryResponse({
        code: HCI_CONNECTION_TIMEOUT,
      });
      await Promise.all([
        fake_peripheral.simulateGATTDisconnection({
          code: HCI_SUCCESS,
        }),
        // Using promise_rejects_dom here rather than
        // assert_promise_rejects_with_message as the race between
        // simulateGATTDisconnection and getPrimaryServices might end up giving
        // slightly different exception message (i.e has "Failed to execute ...
        // on
        // ... " prefix when disconnected state is reflected on the renderer
        // side). The point of the test is no matter how race between them, the
        // promise will be rejected as opposed to get stuck.
        promise_rejects_dom(t, 'NetworkError', device.gatt.CALLS([
          getPrimaryService('health_thermometer') | getPrimaryServices() |
          getPrimaryServices('health_thermometer')[UUID]
        ])),
      ]);
    },
    test_desc, '',
    // As specified above there is a race condition between
    // simulateGATTDisconnection and getPrimaryServices, the artificial
    // GATTDiscoveryResponse might not be consumed in case
    // simulateGATTDisconnection happens first. As a result explicitly skip
    // all response consumed validation at the end of the test.
    /*validate_response_consumed=*/ false);
