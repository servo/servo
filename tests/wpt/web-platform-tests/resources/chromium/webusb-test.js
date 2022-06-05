'use strict';

// This polyfill library implements the WebUSB Test API as specified here:
// https://wicg.github.io/webusb/test/

(() => {

// These variables are logically members of the USBTest class but are defined
// here to hide them from being visible as fields of navigator.usb.test.
let internal = {
  intialized: false,

  webUsbService: null,
  webUsbServiceInterceptor: null,

  messagePort: null,
};

let mojom = {};

async function loadMojomDefinitions() {
  const deviceMojom =
      await import('/gen/services/device/public/mojom/usb_device.mojom.m.js');
  const serviceMojom = await import(
      '/gen/third_party/blink/public/mojom/usb/web_usb_service.mojom.m.js');
  return {
    ...deviceMojom,
    ...serviceMojom,
  };
}

function getMessagePort(target) {
  return new Promise(resolve => {
    target.addEventListener('message', messageEvent => {
      if (messageEvent.data.type === 'ReadyForAttachment') {
        if (internal.messagePort === null) {
          internal.messagePort = messageEvent.data.port;
        }
        resolve();
      }
    }, {once: true});
  });
}

// Converts an ECMAScript String object to an instance of
// mojo_base.mojom.String16.
function mojoString16ToString(string16) {
  return String.fromCharCode.apply(null, string16.data);
}

// Converts an instance of mojo_base.mojom.String16 to an ECMAScript String.
function stringToMojoString16(string) {
  let array = new Array(string.length);
  for (var i = 0; i < string.length; ++i) {
    array[i] = string.charCodeAt(i);
  }
  return { data: array }
}

function fakeDeviceInitToDeviceInfo(guid, init) {
  let deviceInfo = {
    guid: guid + "",
    usbVersionMajor: init.usbVersionMajor,
    usbVersionMinor: init.usbVersionMinor,
    usbVersionSubminor: init.usbVersionSubminor,
    classCode: init.deviceClass,
    subclassCode: init.deviceSubclass,
    protocolCode: init.deviceProtocol,
    vendorId: init.vendorId,
    productId: init.productId,
    deviceVersionMajor: init.deviceVersionMajor,
    deviceVersionMinor: init.deviceVersionMinor,
    deviceVersionSubminor: init.deviceVersionSubminor,
    manufacturerName: stringToMojoString16(init.manufacturerName),
    productName: stringToMojoString16(init.productName),
    serialNumber: stringToMojoString16(init.serialNumber),
    activeConfiguration: init.activeConfigurationValue,
    configurations: []
  };
  init.configurations.forEach(config => {
    var configInfo = {
      configurationValue: config.configurationValue,
      configurationName: stringToMojoString16(config.configurationName),
      selfPowered: false,
      remoteWakeup: false,
      maximumPower: 0,
      interfaces: [],
      extraData: new Uint8Array()
    };
    config.interfaces.forEach(iface => {
      var interfaceInfo = {
        interfaceNumber: iface.interfaceNumber,
        alternates: []
      };
      iface.alternates.forEach(alternate => {
        var alternateInfo = {
          alternateSetting: alternate.alternateSetting,
          classCode: alternate.interfaceClass,
          subclassCode: alternate.interfaceSubclass,
          protocolCode: alternate.interfaceProtocol,
          interfaceName: stringToMojoString16(alternate.interfaceName),
          endpoints: [],
          extraData: new Uint8Array()
        };
        alternate.endpoints.forEach(endpoint => {
          var endpointInfo = {
            endpointNumber: endpoint.endpointNumber,
            packetSize: endpoint.packetSize,
            synchronizationType: mojom.UsbSynchronizationType.NONE,
            usageType: mojom.UsbUsageType.DATA,
            pollingInterval: 0,
            extraData: new Uint8Array()
          };
          switch (endpoint.direction) {
            case "in":
              endpointInfo.direction = mojom.UsbTransferDirection.INBOUND;
              break;
            case "out":
              endpointInfo.direction = mojom.UsbTransferDirection.OUTBOUND;
              break;
          }
          switch (endpoint.type) {
            case "bulk":
              endpointInfo.type = mojom.UsbTransferType.BULK;
              break;
            case "interrupt":
              endpointInfo.type = mojom.UsbTransferType.INTERRUPT;
              break;
            case "isochronous":
              endpointInfo.type = mojom.UsbTransferType.ISOCHRONOUS;
              break;
          }
          alternateInfo.endpoints.push(endpointInfo);
        });
        interfaceInfo.alternates.push(alternateInfo);
      });
      configInfo.interfaces.push(interfaceInfo);
    });
    deviceInfo.configurations.push(configInfo);
  });
  return deviceInfo;
}

function convertMojoDeviceFilters(input) {
  let output = [];
  input.forEach(filter => {
    output.push(convertMojoDeviceFilter(filter));
  });
  return output;
}

function convertMojoDeviceFilter(input) {
  let output = {};
  if (input.hasVendorId)
    output.vendorId = input.vendorId;
  if (input.hasProductId)
    output.productId = input.productId;
  if (input.hasClassCode)
    output.classCode = input.classCode;
  if (input.hasSubclassCode)
    output.subclassCode = input.subclassCode;
  if (input.hasProtocolCode)
    output.protocolCode = input.protocolCode;
  if (input.serialNumber)
    output.serialNumber = mojoString16ToString(input.serialNumber);
  return output;
}

class FakeDevice {
  constructor(deviceInit) {
    this.info_ = deviceInit;
    this.opened_ = false;
    this.currentConfiguration_ = null;
    this.claimedInterfaces_ = new Map();
  }

  getConfiguration() {
    if (this.currentConfiguration_) {
      return Promise.resolve({
          value: this.currentConfiguration_.configurationValue });
    } else {
      return Promise.resolve({ value: 0 });
    }
  }

  open() {
    assert_false(this.opened_);
    this.opened_ = true;
    return Promise.resolve({error: mojom.UsbOpenDeviceError.OK});
  }

  close() {
    assert_true(this.opened_);
    this.opened_ = false;
    return Promise.resolve();
  }

  setConfiguration(value) {
    assert_true(this.opened_);

    let selectedConfiguration = this.info_.configurations.find(
        configuration => configuration.configurationValue == value);
    // Blink should never request an invalid configuration.
    assert_not_equals(selectedConfiguration, undefined);
    this.currentConfiguration_ = selectedConfiguration;
    return Promise.resolve({ success: true });
  }

  async claimInterface(interfaceNumber) {
    assert_true(this.opened_);
    assert_false(this.currentConfiguration_ == null, 'device configured');
    assert_false(this.claimedInterfaces_.has(interfaceNumber),
                 'interface already claimed');

    const protectedInterfaces = new Set([
      mojom.USB_AUDIO_CLASS,
      mojom.USB_HID_CLASS,
      mojom.USB_MASS_STORAGE_CLASS,
      mojom.USB_SMART_CARD_CLASS,
      mojom.USB_VIDEO_CLASS,
      mojom.USB_AUDIO_VIDEO_CLASS,
      mojom.USB_WIRELESS_CLASS,
    ]);

    let iface = this.currentConfiguration_.interfaces.find(
        iface => iface.interfaceNumber == interfaceNumber);
    // Blink should never request an invalid interface or alternate.
    assert_false(iface == undefined);
    if (iface.alternates.some(
            alt => protectedInterfaces.has(alt.interfaceClass))) {
      return {result: mojom.UsbClaimInterfaceResult.kProtectedClass};
    }

    this.claimedInterfaces_.set(interfaceNumber, 0);
    return {result: mojom.UsbClaimInterfaceResult.kSuccess};
  }

  releaseInterface(interfaceNumber) {
    assert_true(this.opened_);
    assert_false(this.currentConfiguration_ == null, 'device configured');
    assert_true(this.claimedInterfaces_.has(interfaceNumber));
    this.claimedInterfaces_.delete(interfaceNumber);
    return Promise.resolve({ success: true });
  }

  setInterfaceAlternateSetting(interfaceNumber, alternateSetting) {
    assert_true(this.opened_);
    assert_false(this.currentConfiguration_ == null, 'device configured');
    assert_true(this.claimedInterfaces_.has(interfaceNumber));

    let iface = this.currentConfiguration_.interfaces.find(
        iface => iface.interfaceNumber == interfaceNumber);
    // Blink should never request an invalid interface or alternate.
    assert_false(iface == undefined);
    assert_true(iface.alternates.some(
        x => x.alternateSetting == alternateSetting));
    this.claimedInterfaces_.set(interfaceNumber, alternateSetting);
    return Promise.resolve({ success: true });
  }

  reset() {
    assert_true(this.opened_);
    return Promise.resolve({ success: true });
  }

  clearHalt(endpoint) {
    assert_true(this.opened_);
    assert_false(this.currentConfiguration_ == null, 'device configured');
    // TODO(reillyg): Assert that endpoint is valid.
    return Promise.resolve({ success: true });
  }

  async controlTransferIn(params, length, timeout) {
    assert_true(this.opened_);

    if ((params.recipient == mojom.UsbControlTransferRecipient.INTERFACE ||
         params.recipient == mojom.UsbControlTransferRecipient.ENDPOINT) &&
        this.currentConfiguration_ == null) {
      return {
        status: mojom.UsbTransferStatus.PERMISSION_DENIED,
      };
    }

    return {
      status: mojom.UsbTransferStatus.OK,
      data: {
        buffer: [
          length >> 8, length & 0xff, params.request, params.value >> 8,
          params.value & 0xff, params.index >> 8, params.index & 0xff
        ]
      }
    };
  }

  async controlTransferOut(params, data, timeout) {
    assert_true(this.opened_);

    if ((params.recipient == mojom.UsbControlTransferRecipient.INTERFACE ||
         params.recipient == mojom.UsbControlTransferRecipient.ENDPOINT) &&
        this.currentConfiguration_ == null) {
      return {
        status: mojom.UsbTransferStatus.PERMISSION_DENIED,
      };
    }

    return {status: mojom.UsbTransferStatus.OK, bytesWritten: data.byteLength};
  }

  genericTransferIn(endpointNumber, length, timeout) {
    assert_true(this.opened_);
    assert_false(this.currentConfiguration_ == null, 'device configured');
    // TODO(reillyg): Assert that endpoint is valid.
    let data = new Array(length);
    for (let i = 0; i < length; ++i)
      data[i] = i & 0xff;
    return Promise.resolve(
        {status: mojom.UsbTransferStatus.OK, data: {buffer: data}});
  }

  genericTransferOut(endpointNumber, data, timeout) {
    assert_true(this.opened_);
    assert_false(this.currentConfiguration_ == null, 'device configured');
    // TODO(reillyg): Assert that endpoint is valid.
    return Promise.resolve(
        {status: mojom.UsbTransferStatus.OK, bytesWritten: data.byteLength});
  }

  isochronousTransferIn(endpointNumber, packetLengths, timeout) {
    assert_true(this.opened_);
    assert_false(this.currentConfiguration_ == null, 'device configured');
    // TODO(reillyg): Assert that endpoint is valid.
    let data = new Array(packetLengths.reduce((a, b) => a + b, 0));
    let dataOffset = 0;
    let packets = new Array(packetLengths.length);
    for (let i = 0; i < packetLengths.length; ++i) {
      for (let j = 0; j < packetLengths[i]; ++j)
        data[dataOffset++] = j & 0xff;
      packets[i] = {
        length: packetLengths[i],
        transferredLength: packetLengths[i],
        status: mojom.UsbTransferStatus.OK
      };
    }
    return Promise.resolve({data: {buffer: data}, packets: packets});
  }

  isochronousTransferOut(endpointNumber, data, packetLengths, timeout) {
    assert_true(this.opened_);
    assert_false(this.currentConfiguration_ == null, 'device configured');
    // TODO(reillyg): Assert that endpoint is valid.
    let packets = new Array(packetLengths.length);
    for (let i = 0; i < packetLengths.length; ++i) {
      packets[i] = {
        length: packetLengths[i],
        transferredLength: packetLengths[i],
        status: mojom.UsbTransferStatus.OK
      };
    }
    return Promise.resolve({ packets: packets });
  }
}

class FakeWebUsbService {
  constructor() {
    this.receiver_ = new mojom.WebUsbServiceReceiver(this);
    this.devices_ = new Map();
    this.devicesByGuid_ = new Map();
    this.client_ = null;
    this.nextGuid_ = 0;
  }

  addBinding(handle) {
    this.receiver_.$.bindHandle(handle);
  }

  addDevice(fakeDevice, info) {
    let device = {
      fakeDevice: fakeDevice,
      guid: (this.nextGuid_++).toString(),
      info: info,
      receivers: [],
    };
    this.devices_.set(fakeDevice, device);
    this.devicesByGuid_.set(device.guid, device);
    if (this.client_)
      this.client_.onDeviceAdded(fakeDeviceInitToDeviceInfo(device.guid, info));
  }

  async forgetDevice(guid) {
    // Permissions are currently untestable through WPT.
  }

  removeDevice(fakeDevice) {
    let device = this.devices_.get(fakeDevice);
    if (!device)
      throw new Error('Cannot remove unknown device.');

    for (const receiver of device.receivers)
      receiver.$.close();
    this.devices_.delete(device.fakeDevice);
    this.devicesByGuid_.delete(device.guid);
    if (this.client_) {
      this.client_.onDeviceRemoved(
          fakeDeviceInitToDeviceInfo(device.guid, device.info));
    }
  }

  removeAllDevices() {
    this.devices_.forEach(device => {
      for (const receiver of device.receivers)
        receiver.$.close();
      this.client_.onDeviceRemoved(
          fakeDeviceInitToDeviceInfo(device.guid, device.info));
    });
    this.devices_.clear();
    this.devicesByGuid_.clear();
  }

  getDevices() {
    let devices = [];
    this.devices_.forEach(device => {
      devices.push(fakeDeviceInitToDeviceInfo(device.guid, device.info));
    });
    return Promise.resolve({ results: devices });
  }

  getDevice(guid, request) {
    let retrievedDevice = this.devicesByGuid_.get(guid);
    if (retrievedDevice) {
      const receiver =
          new mojom.UsbDeviceReceiver(new FakeDevice(retrievedDevice.info));
      receiver.$.bindHandle(request.handle);
      receiver.onConnectionError.addListener(() => {
        if (retrievedDevice.fakeDevice.onclose)
          retrievedDevice.fakeDevice.onclose();
      });
      retrievedDevice.receivers.push(receiver);
    } else {
      request.handle.close();
    }
  }

  getPermission(deviceFilters) {
    return new Promise(resolve => {
      if (navigator.usb.test.onrequestdevice) {
        navigator.usb.test.onrequestdevice(
            new USBDeviceRequestEvent(deviceFilters, resolve));
      } else {
        resolve({ result: null });
      }
    });
  }

  setClient(client) {
    this.client_ = client;
  }
}

class USBDeviceRequestEvent {
  constructor(deviceFilters, resolve) {
    this.filters = convertMojoDeviceFilters(deviceFilters);
    this.resolveFunc_ = resolve;
  }

  respondWith(value) {
    // Wait until |value| resolves (if it is a Promise). This function returns
    // no value.
    Promise.resolve(value).then(fakeDevice => {
      let device = internal.webUsbService.devices_.get(fakeDevice);
      let result = null;
      if (device) {
        result = fakeDeviceInitToDeviceInfo(device.guid, device.info);
      }
      this.resolveFunc_({ result: result });
    }, () => {
      this.resolveFunc_({ result: null });
    });
  }
}

// Unlike FakeDevice this class is exported to callers of USBTest.addFakeDevice.
class FakeUSBDevice {
  constructor() {
    this.onclose = null;
  }

  disconnect() {
    setTimeout(() => internal.webUsbService.removeDevice(this), 0);
  }
}

class USBTest {
  constructor() {
    this.onrequestdevice = undefined;
  }

  async initialize() {
    if (internal.initialized)
      return;

    // Be ready to handle 'ReadyForAttachment' message from child iframes.
    if ('window' in self) {
      getMessagePort(window);
    }

    mojom = await loadMojomDefinitions();
    internal.webUsbService = new FakeWebUsbService();
    internal.webUsbServiceInterceptor =
        new MojoInterfaceInterceptor(mojom.WebUsbService.$interfaceName);
    internal.webUsbServiceInterceptor.oninterfacerequest =
        e => internal.webUsbService.addBinding(e.handle);
    internal.webUsbServiceInterceptor.start();

    // Wait for a call to GetDevices() to pass between the renderer and the
    // mock in order to establish that everything is set up.
    await navigator.usb.getDevices();
    internal.initialized = true;
  }

  // Returns a promise that is resolved when the implementation of |usb| in the
  // global scope for |context| is controlled by the current context.
  attachToContext(context) {
    if (!internal.initialized)
      throw new Error('Call initialize() before attachToContext()');

    let target = context.constructor.name === 'Worker' ? context : window;
    return getMessagePort(target).then(() => {
      return new Promise(resolve => {
        internal.messagePort.onmessage = channelEvent => {
          switch (channelEvent.data.type) {
            case mojom.WebUsbService.$interfaceName:
              internal.webUsbService.addBinding(channelEvent.data.handle);
              break;
            case 'Complete':
              resolve();
              break;
          }
        };
        internal.messagePort.postMessage({
          type: 'Attach',
          interfaces: [
            mojom.WebUsbService.$interfaceName,
          ]
        });
      });
    });
  }

  addFakeDevice(deviceInit) {
    if (!internal.initialized)
      throw new Error('Call initialize() before addFakeDevice().');

    // |addDevice| and |removeDevice| are called in a setTimeout callback so
    // that tests do not rely on the device being immediately available which
    // may not be true for all implementations of this test API.
    let fakeDevice = new FakeUSBDevice();
    setTimeout(
        () => internal.webUsbService.addDevice(fakeDevice, deviceInit), 0);
    return fakeDevice;
  }

  reset() {
    if (!internal.initialized)
      throw new Error('Call initialize() before reset().');

    // Reset the mocks in a setTimeout callback so that tests do not rely on
    // the fact that this polyfill can do this synchronously.
    return new Promise(resolve => {
      setTimeout(() => {
        if (internal.messagePort !== null)
          internal.messagePort.close();
        internal.messagePort = null;
        internal.webUsbService.removeAllDevices();
        resolve();
      }, 0);
    });
  }
}

navigator.usb.test = new USBTest();

})();
