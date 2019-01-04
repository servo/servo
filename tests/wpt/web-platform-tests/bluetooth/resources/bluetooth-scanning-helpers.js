'use strict';

const company_id = '224';
const data = new TextEncoder().encode('foo');
const manufacturerDataMap = {[company_id]: data};
const health_uuid = health_thermometer.uuid;
const serviceDataMap = {[health_uuid]: data};
const scanRecord = {
  name: 'Health Thermometer',
  uuids: ['generic_access', health_uuid],
  txPower: 20,
  appearance: 100,
  manufacturerData: manufacturerDataMap,
  serviceData: serviceDataMap,
};
const scanResult = {
  deviceAddress: '09:09:09:09:09:09',
  rssi: 100,
  scanRecord: scanRecord,
};

function verifyBluetoothAdvertisingEvent(e) {
  assert_equals(e.constructor.name, 'BluetoothAdvertisingEvent')
  assert_equals(e.device.name, scanRecord.name)
  assert_equals(e.name, scanRecord.name)
  assert_array_equals(e.uuids,
    ["00001800-0000-1000-8000-00805f9b34fb",
     "00001809-0000-1000-8000-00805f9b34fb"])
  assert_equals(e.txPower, 20)
  assert_equals(e.rssi, 100)

  assert_equals(e.manufacturerData.constructor.name,
                'BluetoothManufacturerDataMap')
  assert_equals(data[0], e.manufacturerData.get(224).getUint8(0))
  assert_equals(data[1], e.manufacturerData.get(224).getUint8(1))
  assert_equals(data[2], e.manufacturerData.get(224).getUint8(2))

  assert_equals(e.serviceData.constructor.name, 'BluetoothServiceDataMap')
  assert_equals(data[0], e.serviceData.get(health_uuid).getUint8(0))
  assert_equals(data[1], e.serviceData.get(health_uuid).getUint8(1))
  assert_equals(data[2], e.serviceData.get(health_uuid).getUint8(2))
}