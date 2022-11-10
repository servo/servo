// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Matches a filter if all present members match.';
let matching_services = [health_thermometer.uuid];
let matching_name = 'Health Thermometer';
let matching_namePrefix = 'Health';
let matching_manufacturerData = [{companyIdentifier: 0x0001}];

let test_specs = [
  {
    filters: [{
      services: matching_services,
    }]
  },
  {
    filters: [{
      services: matching_services,
      name: matching_name,
    }]
  },
  {filters: [{services: matching_services, namePrefix: matching_namePrefix}]}, {
    filters: [
      {services: matching_services, manufacturerData: matching_manufacturerData}
    ]
  },
  {
    filters: [{
      name: matching_name,
    }],
    optionalServices: matching_services
  },
  {
    filters: [{namePrefix: matching_namePrefix}],
    optionalServices: matching_services
  },
  {
    filters: [{manufacturerData: matching_manufacturerData}],
    optionalServices: matching_services
  },
  {
    filters: [{
      name: matching_name,
      namePrefix: matching_namePrefix,
      manufacturerData: matching_manufacturerData
    }],
    optionalServices: matching_services
  },
  {
    filters: [{
      services: matching_services,
      name: matching_name,
      namePrefix: matching_namePrefix,
      manufacturerData: matching_manufacturerData
    }]
  }
];

bluetooth_test(
    () => setUpHealthThermometerDevice().then(() => {
      let test_promises = Promise.resolve();
      test_specs.forEach(args => {
        test_promises =
            test_promises.then(() => requestDeviceWithTrustedClick(args))
                .then(device => {
                  // We always have access to the services in matching_services
                  // because we include them in a filter or in optionalServices.
                  assert_equals(device.name, matching_name);
                  assert_true(device.name.startsWith(matching_namePrefix));
                });
      });
      return test_promises;
    }),
    test_desc);
