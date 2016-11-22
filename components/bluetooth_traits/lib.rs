/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(proc_macro)]

extern crate ipc_channel;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate util;

pub mod blocklist;
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

pub type BluetoothResponseResult = Result<BluetoothResponse, BluetoothError>;

#[derive(Deserialize, Serialize)]
pub enum BluetoothRequest {
    RequestDevice(RequestDeviceoptions, IpcSender<BluetoothResponseResult>),
    GATTServerConnect(String, IpcSender<BluetoothResponseResult>),
    GATTServerDisconnect(String, IpcSender<BluetoothResult<bool>>),
    GetPrimaryService(String, String, IpcSender<BluetoothResponseResult>),
    GetPrimaryServices(String, Option<String>, IpcSender<BluetoothResponseResult>),
    GetIncludedService(String, String, IpcSender<BluetoothResponseResult>),
    GetIncludedServices(String, Option<String>, IpcSender<BluetoothResponseResult>),
    GetCharacteristic(String, String, IpcSender<BluetoothResponseResult>),
    GetCharacteristics(String, Option<String>, IpcSender<BluetoothResponseResult>),
    GetDescriptor(String, String, IpcSender<BluetoothResponseResult>),
    GetDescriptors(String, Option<String>, IpcSender<BluetoothResponseResult>),
    ReadValue(String, IpcSender<BluetoothResponseResult>),
    WriteValue(String, Vec<u8>, IpcSender<BluetoothResponseResult>),
    EnableNotification(String, bool, IpcSender<BluetoothResponseResult>),
    Test(String, IpcSender<BluetoothResult<()>>),
    Exit,
}

#[derive(Deserialize, Serialize)]
pub enum BluetoothResponse {
    RequestDevice(BluetoothDeviceMsg),
    GATTServerConnect(bool),
    GetPrimaryService(BluetoothServiceMsg),
    GetPrimaryServices(BluetoothServicesMsg),
    GetIncludedService(BluetoothServiceMsg),
    GetIncludedServices(BluetoothServicesMsg),
    GetCharacteristic(BluetoothCharacteristicMsg),
    GetCharacteristics(BluetoothCharacteristicsMsg),
    GetDescriptor(BluetoothDescriptorMsg),
    GetDescriptors(BluetoothDescriptorsMsg),
    ReadValue(Vec<u8>),
    WriteValue(Vec<u8>),
    EnableNotification(()),
}

pub trait BluetoothResponseListener {
    fn response(&mut self, response: BluetoothResponseResult);
}
