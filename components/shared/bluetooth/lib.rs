/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod blocklist;
pub mod scanfilter;

use base::generic_channel::{GenericCallback, GenericSender};
use serde::{Deserialize, Serialize};

use crate::scanfilter::{BluetoothScanfilterSequence, RequestDeviceoptions};

#[derive(Debug, Deserialize, Serialize)]
pub enum BluetoothError {
    Type(String),
    Network,
    NotFound,
    NotSupported,
    Security,
    InvalidState,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum GATTType {
    PrimaryService,
    Characteristic,
    IncludedService,
    Descriptor,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BluetoothDeviceMsg {
    // Bluetooth Device properties
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BluetoothServiceMsg {
    pub uuid: String,
    pub is_primary: bool,
    pub instance_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct BluetoothDescriptorMsg {
    pub uuid: String,
    pub instance_id: String,
}

pub type BluetoothServicesMsg = Vec<BluetoothServiceMsg>;

pub type BluetoothCharacteristicsMsg = Vec<BluetoothCharacteristicMsg>;

pub type BluetoothDescriptorsMsg = Vec<BluetoothDescriptorMsg>;

pub type BluetoothResult<T> = Result<T, BluetoothError>;

pub type BluetoothResponseResult = Result<BluetoothResponse, BluetoothError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum BluetoothRequest {
    RequestDevice(
        RequestDeviceoptions,
        GenericCallback<BluetoothResponseResult>,
    ),
    GATTServerConnect(String, GenericCallback<BluetoothResponseResult>),
    GATTServerDisconnect(String, GenericSender<BluetoothResult<()>>),
    GetGATTChildren(
        String,
        Option<String>,
        bool,
        GATTType,
        GenericCallback<BluetoothResponseResult>,
    ),
    ReadValue(String, GenericCallback<BluetoothResponseResult>),
    WriteValue(String, Vec<u8>, GenericCallback<BluetoothResponseResult>),
    EnableNotification(String, bool, GenericCallback<BluetoothResponseResult>),
    WatchAdvertisements(String, GenericCallback<BluetoothResponseResult>),
    SetRepresentedToNull(Vec<String>, Vec<String>, Vec<String>),
    IsRepresentedDeviceNull(String, GenericSender<bool>),
    GetAvailability(GenericCallback<BluetoothResponseResult>),
    MatchesFilter(
        String,
        BluetoothScanfilterSequence,
        GenericSender<BluetoothResult<bool>>,
    ),
    Test(String, GenericSender<BluetoothResult<()>>),
    Exit,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum BluetoothResponse {
    RequestDevice(BluetoothDeviceMsg),
    GATTServerConnect(bool),
    GetPrimaryServices(BluetoothServicesMsg, bool),
    GetIncludedServices(BluetoothServicesMsg, bool),
    GetCharacteristics(BluetoothCharacteristicsMsg, bool),
    GetDescriptors(BluetoothDescriptorsMsg, bool),
    ReadValue(Vec<u8>),
    WriteValue(Vec<u8>),
    EnableNotification(()),
    WatchAdvertisements(()),
    GetAvailability(bool),
}
