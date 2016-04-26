/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use bluetooth_scanfilter::RequestDeviceoptions;
use ipc_channel::ipc::IpcSender;

pub type BluetoothResult<T> = Result<T, String>;

pub type BluetoothDeviceMsg = (// Bluetooth Device properties
                               String, Option<String>, Option<u32>, Option<String>,
                               Option<u32>, Option<u32>, Option<u32>,
                               // Advertisiong Data properties
                               Option<u16>, Option<i8>, Option<i8>);

pub type BluetoothServiceMsg = (String, bool, String);

pub type BluetoothServicesMsg = Vec<BluetoothServiceMsg>;

pub type BluetoothCharacteristicMsg = (// Characteristic
                                       String, String,
                                       // Characteristic properties
                                       bool, bool, bool, bool, bool, bool, bool, bool, bool);

pub type BluetoothCharacteristicsMsg = Vec<BluetoothCharacteristicMsg>;

pub type BluetoothDescriptorMsg = (String, String);

pub type BluetoothDescriptorsMsg = Vec<BluetoothDescriptorMsg>;

#[derive(Deserialize, Serialize)]
pub enum BluetoothMethodMsg {
    RequestDevice(RequestDeviceoptions, IpcSender<BluetoothResult<BluetoothDeviceMsg>>),
    GATTServerConnect(String, IpcSender<BluetoothResult<bool>>),
    GATTServerDisconnect(String, IpcSender<BluetoothResult<bool>>),
    GetPrimaryService(String, String, IpcSender<BluetoothResult<BluetoothServiceMsg>>),
    GetPrimaryServices(String, Option<String>, IpcSender<BluetoothResult<BluetoothServicesMsg>>),
    GetCharacteristic(String, String, IpcSender<BluetoothResult<BluetoothCharacteristicMsg>>),
    GetCharacteristics(String, Option<String>, IpcSender<BluetoothResult<BluetoothCharacteristicsMsg>>),
    GetDescriptor(String, String, IpcSender<BluetoothResult<BluetoothDescriptorMsg>>),
    GetDescriptors(String, Option<String>, IpcSender<BluetoothResult<BluetoothDescriptorsMsg>>),
    ReadValue(String, IpcSender<BluetoothResult<Vec<u8>>>),
    WriteValue(String, Vec<u8>, IpcSender<BluetoothResult<bool>>),
    Exit,
}
