/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(proc_macro)]

extern crate ipc_channel;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate util;

pub mod blacklist;
pub mod scanfilter;

use ipc_channel::ipc::IpcSender;
use scanfilter::RequestDeviceoptions;

#[derive(Deserialize, Serialize)]
pub enum BluetoothError {
    Type(String),
    Network,
    NotFound,
    NotSupported,
    Security,
    InvalidState,
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothDeviceMsg {
    // Bluetooth Device properties
    pub id: String,
    pub name: Option<String>,
    // Advertisiong Data properties
    pub appearance: Option<u16>,
    pub tx_power: Option<i8>,
    pub rssi: Option<i8>,
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothServiceMsg {
    pub uuid: String,
    pub is_primary: bool,
    pub instance_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothCharacteristicMsg {
    // Characteristic
    pub uuid: String,
    pub instance_id: String,
    // Characteristic properties
    pub broadcast: bool,
    pub read: bool,
    pub write_without_response: bool,
    pub write: bool,
    pub notify: bool,
    pub indicate: bool,
    pub authenticated_signed_writes: bool,
    pub reliable_write: bool,
    pub writable_auxiliaries: bool,
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothDescriptorMsg {
    pub uuid: String,
    pub instance_id: String,
}

pub type BluetoothServicesMsg = Vec<BluetoothServiceMsg>;

pub type BluetoothCharacteristicsMsg = Vec<BluetoothCharacteristicMsg>;

pub type BluetoothDescriptorsMsg = Vec<BluetoothDescriptorMsg>;

pub type BluetoothResult<T> = Result<T, BluetoothError>;

#[derive(Deserialize, Serialize)]
pub enum BluetoothMethodMsg {
    RequestDevice(RequestDeviceoptions, IpcSender<BluetoothResult<BluetoothDeviceMsg>>),
    GATTServerConnect(String, IpcSender<BluetoothResult<bool>>),
    GATTServerDisconnect(String, IpcSender<BluetoothResult<bool>>),
    GetPrimaryService(String, String, IpcSender<BluetoothResult<BluetoothServiceMsg>>),
    GetPrimaryServices(String, Option<String>, IpcSender<BluetoothResult<BluetoothServicesMsg>>),
    GetIncludedService(String, String, IpcSender<BluetoothResult<BluetoothServiceMsg>>),
    GetIncludedServices(String, Option<String>, IpcSender<BluetoothResult<BluetoothServicesMsg>>),
    GetCharacteristic(String, String, IpcSender<BluetoothResult<BluetoothCharacteristicMsg>>),
    GetCharacteristics(String, Option<String>, IpcSender<BluetoothResult<BluetoothCharacteristicsMsg>>),
    GetDescriptor(String, String, IpcSender<BluetoothResult<BluetoothDescriptorMsg>>),
    GetDescriptors(String, Option<String>, IpcSender<BluetoothResult<BluetoothDescriptorsMsg>>),
    ReadValue(String, IpcSender<BluetoothResult<Vec<u8>>>),
    WriteValue(String, Vec<u8>, IpcSender<BluetoothResult<bool>>),
    Test(String, IpcSender<BluetoothResult<()>>),
    Exit,
}
