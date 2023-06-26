'use strict';

let fakeDeviceInit = {
  usbVersionMajor: 2,
  usbVersionMinor: 0,
  usbVersionSubminor: 0,
  deviceClass: 7,
  deviceSubclass: 1,
  deviceProtocol: 2,
  vendorId: 0x18d1,
  productId: 0xf00d,
  deviceVersionMajor: 1,
  deviceVersionMinor: 2,
  deviceVersionSubminor: 3,
  manufacturerName: 'Google, Inc.',
  productName: 'The amazing imaginary printer',
  serialNumber: '4',
  activeConfigurationValue: 0,
  configurations: [
    {
      configurationValue: 1,
      configurationName: 'Printer Mode',
      interfaces: [
        {
          interfaceNumber: 0,
          alternates: [{
            alternateSetting: 0,
            interfaceClass: 0xff,
            interfaceSubclass: 0x01,
            interfaceProtocol: 0x01,
            interfaceName: 'Control',
            endpoints: [{
              endpointNumber: 1,
              direction: 'in',
              type: 'interrupt',
              packetSize: 8
            }]
          }]
        },
        {
          interfaceNumber: 1,
          alternates: [{
            alternateSetting: 0,
            interfaceClass: 0xff,
            interfaceSubclass: 0x02,
            interfaceProtocol: 0x01,
            interfaceName: 'Data',
            endpoints: [
              {
                endpointNumber: 2,
                direction: 'in',
                type: 'bulk',
                packetSize: 1024
              },
              {
                endpointNumber: 2,
                direction: 'out',
                type: 'bulk',
                packetSize: 1024
              }
            ]
          }]
        }
      ]
    },
    {
      configurationValue: 2,
      configurationName: 'Fighting Robot Mode',
      interfaces: [{
        interfaceNumber: 0,
        alternates: [
          {
            alternateSetting: 0,
            interfaceClass: 0xff,
            interfaceSubclass: 0x42,
            interfaceProtocol: 0x01,
            interfaceName: 'Disabled',
            endpoints: []
          },
          {
            alternateSetting: 1,
            interfaceClass: 0xff,
            interfaceSubclass: 0x42,
            interfaceProtocol: 0x01,
            interfaceName: 'Activate!',
            endpoints: [
              {
                endpointNumber: 1,
                direction: 'in',
                type: 'isochronous',
                packetSize: 1024
              },
              {
                endpointNumber: 1,
                direction: 'out',
                type: 'isochronous',
                packetSize: 1024
              }
            ]
          }
        ]
      }]
    },
    {
      configurationValue: 3,
      configurationName: 'Non-sequential interface number and alternate ' +
          'setting Mode',
      interfaces: [
        {
          interfaceNumber: 0,
          alternates: [
            {
              alternateSetting: 0,
              interfaceClass: 0xff,
              interfaceSubclass: 0x01,
              interfaceProtocol: 0x01,
              interfaceName: 'Control',
              endpoints: [{
                endpointNumber: 1,
                direction: 'in',
                type: 'interrupt',
                packetSize: 8
              }]
            },
            {
              alternateSetting: 2,
              interfaceClass: 0xff,
              interfaceSubclass: 0x02,
              interfaceProtocol: 0x01,
              interfaceName: 'Data',
              endpoints: [
                {
                  endpointNumber: 2,
                  direction: 'in',
                  type: 'bulk',
                  packetSize: 1024
                },
                {
                  endpointNumber: 2,
                  direction: 'out',
                  type: 'bulk',
                  packetSize: 1024
                }
              ]
            }
          ]
        },
        {
          interfaceNumber: 2,
          alternates: [{
            alternateSetting: 0,
            interfaceClass: 0xff,
            interfaceSubclass: 0x02,
            interfaceProtocol: 0x01,
            interfaceName: 'Data',
            endpoints: [
              {
                endpointNumber: 2,
                direction: 'in',
                type: 'bulk',
                packetSize: 1024
              },
              {
                endpointNumber: 2,
                direction: 'out',
                type: 'bulk',
                packetSize: 1024
              }
            ]
          }]
        }
      ]
    }
  ]
};
