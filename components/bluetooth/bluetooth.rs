/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

#[cfg(feature = "bluetooth-test")]
use blurmock::fake_characteristic::FakeBluetoothGATTCharacteristic;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_descriptor::FakeBluetoothGATTDescriptor;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_device::FakeBluetoothDevice;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_discovery_session::FakeBluetoothDiscoverySession;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_service::FakeBluetoothGATTService;
#[cfg(feature = "native-bluetooth")]
use btleplug::api::{Central, CharPropFlags, Peripheral, ScanFilter, WriteType};
#[cfg(feature = "native-bluetooth")]
use btleplug::platform::{Adapter, Peripheral as PlatformPeripheral};

pub use super::adapter::BluetoothAdapter;
use crate::macros::get_inner_and_call_test_func;

#[cfg(feature = "native-bluetooth")]
#[derive(Clone, Debug)]
pub struct BtleplugDiscoverySession {
    pub(crate) adapter: Adapter,
}

#[cfg(feature = "native-bluetooth")]
impl BtleplugDiscoverySession {
    pub async fn start_discovery(&self) -> Result<(), Box<dyn Error>> {
        Ok(self.adapter.start_scan(ScanFilter::default()).await?)
    }

    pub async fn stop_discovery(&self) -> Result<(), Box<dyn Error>> {
        Ok(self.adapter.stop_scan().await?)
    }
}

#[cfg(feature = "native-bluetooth")]
#[derive(Clone, Debug)]
pub struct BtleplugDevice {
    pub(crate) peripheral: PlatformPeripheral,
}

#[cfg(feature = "native-bluetooth")]
impl BtleplugDevice {
    async fn properties(&self) -> Result<btleplug::api::PeripheralProperties, Box<dyn Error>> {
        self.peripheral
            .properties()
            .await?
            .ok_or_else(|| Box::from("Device properties not available"))
    }

    pub fn get_id(&self) -> String {
        self.peripheral.id().to_string()
    }

    pub fn get_address(&self) -> Result<String, Box<dyn Error>> {
        // On macOS, CoreBluetooth doesn't expose real MAC addresses but always returns 00:00:00:00:00:00.
        // Use the peripheral ID (a UUID) as a unique identifier instead.
        Ok(self.peripheral.id().to_string())
    }

    pub async fn get_name(&self) -> Result<String, Box<dyn Error>> {
        let props = self.properties().await?;
        if let Some(name) = props.local_name {
            return Ok(name);
        }
        if let Some(name) = props.advertisement_name {
            return Ok(name);
        }
        Err(Box::from("Device name not available"))
    }

    pub async fn get_uuids(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self
            .properties()
            .await?
            .services
            .into_iter()
            .map(|uuid| uuid.to_string())
            .collect())
    }

    pub async fn is_connected(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.peripheral.is_connected().await.map_err(Box::new)?)
    }

    pub async fn connect(&self) -> Result<(), Box<dyn Error>> {
        Ok(self.peripheral.connect().await.map_err(Box::new)?)
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn Error>> {
        Ok(self.peripheral.disconnect().await.map_err(Box::new)?)
    }

    pub async fn get_manufacturer_data(&self) -> Result<HashMap<u16, Vec<u8>>, Box<dyn Error>> {
        Ok(self.properties().await?.manufacturer_data)
    }

    pub async fn get_service_data(&self) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
        Ok(self
            .properties()
            .await?
            .service_data
            .into_iter()
            .map(|(uuid, data)| (uuid.to_string(), data))
            .collect())
    }

    pub async fn discover_services(&self) -> Result<Vec<BluetoothGATTService>, Box<dyn Error>> {
        self.peripheral
            .discover_services()
            .await
            .map_err(Box::new)?;

        let device_id = self.peripheral.id().to_string();
        let services = self.peripheral.services();
        let mut result = Vec::new();
        let mut uuid_counts: HashMap<String, usize> = HashMap::new();

        for service in services {
            let uuid_str = service.uuid.to_string();
            let idx = uuid_counts.entry(uuid_str.clone()).or_default();
            let instance_id = format!("{device_id}/svc/{uuid_str}/{idx}");
            *idx += 1;
            result.push(BluetoothGATTService::Btleplug(BtleplugGATTService {
                instance_id,
                service,
                peripheral: self.peripheral.clone(),
            }));
        }
        Ok(result)
    }
}

#[cfg(feature = "native-bluetooth")]
#[derive(Clone, Debug)]
pub struct BtleplugGATTService {
    pub(crate) instance_id: String,
    pub(crate) service: btleplug::api::Service,
    pub(crate) peripheral: PlatformPeripheral,
}

#[cfg(feature = "native-bluetooth")]
impl BtleplugGATTService {
    pub fn get_id(&self) -> String {
        self.instance_id.clone()
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.service.uuid.to_string())
    }

    pub fn is_primary(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.service.primary)
    }

    pub fn get_includes(&self) -> Result<Vec<String>, Box<dyn Error>> {
        // TODO: btleplug does not support included services yet.
        //       Add support in btleplug upstream.
        Ok(vec![])
    }

    pub fn get_gatt_characteristics(&self) -> Vec<BluetoothGATTCharacteristic> {
        let mut result = Vec::new();
        let mut uuid_counts: HashMap<String, usize> = HashMap::new();

        for characteristic in &self.service.characteristics {
            let uuid_str = characteristic.uuid.to_string();
            let idx = uuid_counts.entry(uuid_str.clone()).or_default();
            let instance_id = format!("{}/char/{uuid_str}/{idx}", self.instance_id);
            *idx += 1;
            result.push(BluetoothGATTCharacteristic::Btleplug(
                BtleplugGATTCharacteristic {
                    instance_id,
                    characteristic: characteristic.clone(),
                    peripheral: self.peripheral.clone(),
                },
            ));
        }
        result
    }
}

#[cfg(feature = "native-bluetooth")]
#[derive(Clone, Debug)]
pub struct BtleplugGATTCharacteristic {
    pub(crate) instance_id: String,
    pub(crate) characteristic: btleplug::api::Characteristic,
    pub(crate) peripheral: PlatformPeripheral,
}

#[cfg(feature = "native-bluetooth")]
impl BtleplugGATTCharacteristic {
    pub fn get_id(&self) -> String {
        self.instance_id.clone()
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.characteristic.uuid.to_string())
    }

    pub fn get_flags(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let props = self.characteristic.properties;
        let mut flags = Vec::new();

        if props.contains(CharPropFlags::BROADCAST) {
            flags.push("broadcast".to_string());
        }
        if props.contains(CharPropFlags::READ) {
            flags.push("read".to_string());
        }
        if props.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) {
            flags.push("write-without-response".to_string());
        }
        if props.contains(CharPropFlags::WRITE) {
            flags.push("write".to_string());
        }
        if props.contains(CharPropFlags::NOTIFY) {
            flags.push("notify".to_string());
        }
        if props.contains(CharPropFlags::INDICATE) {
            flags.push("indicate".to_string());
        }
        if props.contains(CharPropFlags::AUTHENTICATED_SIGNED_WRITES) {
            flags.push("authenticated-signed-writes".to_string());
        }
        Ok(flags)
    }

    pub fn get_gatt_descriptors(&self) -> Vec<BluetoothGATTDescriptor> {
        let mut result = Vec::new();
        let mut uuid_counts: HashMap<String, usize> = HashMap::new();

        for descriptor in &self.characteristic.descriptors {
            let uuid_str = descriptor.uuid.to_string();
            let idx = uuid_counts.entry(uuid_str.clone()).or_default();
            let instance_id = format!("{}/desc/{uuid_str}/{idx}", self.instance_id);
            *idx += 1;
            result.push(BluetoothGATTDescriptor::Btleplug(BtleplugGATTDescriptor {
                instance_id,
                descriptor: descriptor.clone(),
                peripheral: self.peripheral.clone(),
            }));
        }
        result
    }

    pub async fn read_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self
            .peripheral
            .read(&self.characteristic)
            .await
            .map_err(Box::new)?)
    }

    pub async fn write_value(&self, values: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let write_type = if self
            .characteristic
            .properties
            .contains(CharPropFlags::WRITE)
        {
            WriteType::WithResponse
        } else {
            WriteType::WithoutResponse
        };
        Ok(self
            .peripheral
            .write(&self.characteristic, &values, write_type)
            .await
            .map_err(Box::new)?)
    }

    pub async fn start_notify(&self) -> Result<(), Box<dyn Error>> {
        Ok(self
            .peripheral
            .subscribe(&self.characteristic)
            .await
            .map_err(Box::new)?)
    }

    pub async fn stop_notify(&self) -> Result<(), Box<dyn Error>> {
        Ok(self
            .peripheral
            .unsubscribe(&self.characteristic)
            .await
            .map_err(Box::new)?)
    }
}

#[cfg(feature = "native-bluetooth")]
#[derive(Clone, Debug)]
pub struct BtleplugGATTDescriptor {
    pub(crate) instance_id: String,
    pub(crate) descriptor: btleplug::api::Descriptor,
    pub(crate) peripheral: PlatformPeripheral,
}

#[cfg(feature = "native-bluetooth")]
impl BtleplugGATTDescriptor {
    pub fn get_id(&self) -> String {
        self.instance_id.clone()
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.descriptor.uuid.to_string())
    }

    pub fn get_flags(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(vec![])
    }

    pub async fn read_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self
            .peripheral
            .read_descriptor(&self.descriptor)
            .await
            .map_err(Box::new)?)
    }

    pub async fn write_value(&self, values: Vec<u8>) -> Result<(), Box<dyn Error>> {
        Ok(self
            .peripheral
            .write_descriptor(&self.descriptor, &values)
            .await
            .map_err(Box::new)?)
    }
}

#[derive(Debug)]
pub enum BluetoothDiscoverySession {
    #[cfg(feature = "native-bluetooth")]
    Btleplug(BtleplugDiscoverySession),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothDiscoverySession>),
}

impl BluetoothDiscoverySession {
    pub async fn start_discovery(&self) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.start_discovery().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.start_discovery(),
        }
    }

    pub async fn stop_discovery(&self) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.stop_discovery().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.stop_discovery(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BluetoothDevice {
    #[cfg(feature = "native-bluetooth")]
    Btleplug(BtleplugDevice),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothDevice>),
}

impl BluetoothDevice {
    pub fn get_id(&self) -> String {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_id(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_id(),
        }
    }

    pub fn get_address(&self) -> Result<String, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_address(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_address(),
        }
    }

    pub async fn get_name(&self) -> Result<String, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_name().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_name(),
        }
    }

    pub async fn get_uuids(&self) -> Result<Vec<String>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_uuids().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_uuids(),
        }
    }

    pub async fn is_connected(&self) -> Result<bool, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.is_connected().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.is_connected(),
        }
    }

    pub async fn connect(&self) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.connect().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.connect(),
        }
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.disconnect().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.disconnect(),
        }
    }

    pub async fn get_manufacturer_data(&self) -> Result<HashMap<u16, Vec<u8>>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_manufacturer_data().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_manufacturer_data(),
        }
    }

    pub async fn get_service_data(&self) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_service_data().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_service_data(),
        }
    }

    pub async fn get_gatt_services(&self) -> Result<Vec<BluetoothGATTService>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.discover_services().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => {
                let services = inner.get_gatt_services()?;
                Ok(services
                    .into_iter()
                    .map(|service| {
                        BluetoothGATTService::Mock(FakeBluetoothGATTService::new_empty(
                            inner.clone(),
                            service,
                        ))
                    })
                    .collect())
            },
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_id(&self, id: String) {
        #[allow(irrefutable_let_patterns)]
        let Self::Mock(inner) = self else {
            return;
        };
        inner.set_id(id);
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_address(&self, address: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_address, address)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_name(&self, name: Option<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_name, name)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_uuids(&self, uuids: Vec<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_uuids, uuids)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_connectable(&self, connectable: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_connectable, connectable)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_connected(&self, connected: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_connected, connected)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_manufacturer_data(
        &self,
        manufacturer_data: HashMap<u16, Vec<u8>>,
    ) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(
            self,
            BluetoothDevice,
            set_manufacturer_data,
            Some(manufacturer_data)
        )
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_service_data(
        &self,
        service_data: HashMap<String, Vec<u8>>,
    ) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_service_data, Some(service_data))
    }
}

#[derive(Clone, Debug)]
pub enum BluetoothGATTService {
    #[cfg(feature = "native-bluetooth")]
    Btleplug(BtleplugGATTService),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothGATTService>),
}

impl BluetoothGATTService {
    fn create_service(device: BluetoothDevice, service: String) -> BluetoothGATTService {
        match device {
            #[cfg(feature = "native-bluetooth")]
            BluetoothDevice::Btleplug(_) => {
                unreachable!("btleplug services are created directly, not via create_service")
            },
            #[cfg(feature = "bluetooth-test")]
            BluetoothDevice::Mock(fake_device) => BluetoothGATTService::Mock(
                FakeBluetoothGATTService::new_empty(fake_device, service),
            ),
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn create_mock_service(
        device: BluetoothDevice,
        service: String,
    ) -> Result<BluetoothGATTService, Box<dyn Error>> {
        match device {
            BluetoothDevice::Mock(fake_device) => Ok(BluetoothGATTService::Mock(
                FakeBluetoothGATTService::new_empty(fake_device, service),
            )),
            #[cfg(feature = "native-bluetooth")]
            _ => Err(Box::from("The first parameter must be a mock structure")),
        }
    }

    pub fn get_id(&self) -> String {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_id(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_id(),
        }
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_uuid(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_uuid(),
        }
    }

    pub fn is_primary(&self) -> Result<bool, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.is_primary(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.is_primary(),
        }
    }

    pub fn get_includes(
        &self,
        device: BluetoothDevice,
    ) -> Result<Vec<BluetoothGATTService>, Box<dyn Error>> {
        let services = match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_includes()?,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_includes()?,
        };
        Ok(services
            .into_iter()
            .map(|service| BluetoothGATTService::create_service(device.clone(), service))
            .collect())
    }

    pub fn get_gatt_characteristics(
        &self,
    ) -> Result<Vec<BluetoothGATTCharacteristic>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => Ok(inner.get_gatt_characteristics()),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => {
                let characteristics = inner.get_gatt_characteristics()?;
                Ok(characteristics
                    .into_iter()
                    .map(|characteristic| {
                        BluetoothGATTCharacteristic::Mock(
                            FakeBluetoothGATTCharacteristic::new_empty(
                                inner.clone(),
                                characteristic,
                            ),
                        )
                    })
                    .collect())
            },
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_id(&self, id: String) {
        #[allow(irrefutable_let_patterns)]
        let Self::Mock(inner) = self else {
            return;
        };
        inner.set_id(id);
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_uuid(&self, uuid: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTService, set_uuid, uuid)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_primary(&self, primary: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTService, set_is_primary, primary)
    }
}

#[derive(Clone, Debug)]
pub enum BluetoothGATTCharacteristic {
    #[cfg(feature = "native-bluetooth")]
    Btleplug(BtleplugGATTCharacteristic),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothGATTCharacteristic>),
}

impl BluetoothGATTCharacteristic {
    #[cfg(feature = "bluetooth-test")]
    pub fn create_mock_characteristic(
        service: BluetoothGATTService,
        characteristic: String,
    ) -> Result<BluetoothGATTCharacteristic, Box<dyn Error>> {
        match service {
            BluetoothGATTService::Mock(fake_service) => Ok(BluetoothGATTCharacteristic::Mock(
                FakeBluetoothGATTCharacteristic::new_empty(fake_service, characteristic),
            )),
            #[cfg(feature = "native-bluetooth")]
            _ => Err(Box::from("The first parameter must be a mock structure")),
        }
    }

    pub fn get_id(&self) -> String {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_id(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_id(),
        }
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_uuid(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_uuid(),
        }
    }

    pub fn get_flags(&self) -> Result<Vec<String>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_flags(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_flags(),
        }
    }

    pub fn get_gatt_descriptors(&self) -> Result<Vec<BluetoothGATTDescriptor>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => Ok(inner.get_gatt_descriptors()),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => {
                let descriptors = inner.get_gatt_descriptors()?;
                Ok(descriptors
                    .into_iter()
                    .map(|descriptor| {
                        BluetoothGATTDescriptor::Mock(FakeBluetoothGATTDescriptor::new_empty(
                            inner.clone(),
                            descriptor,
                        ))
                    })
                    .collect())
            },
        }
    }

    pub async fn read_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.read_value().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.read_value(),
        }
    }

    pub async fn write_value(&self, values: Vec<u8>) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.write_value(values).await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.write_value(values),
        }
    }

    pub async fn start_notify(&self) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.start_notify().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.start_notify(),
        }
    }

    pub async fn stop_notify(&self) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.stop_notify().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.stop_notify(),
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_id(&self, id: String) {
        #[allow(irrefutable_let_patterns)]
        let Self::Mock(inner) = self else {
            return;
        };
        inner.set_id(id);
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_uuid(&self, uuid: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTCharacteristic, set_uuid, uuid)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_value(&self, value: Vec<u8>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTCharacteristic, set_value, Some(value))
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_flags(&self, flags: Vec<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTCharacteristic, set_flags, flags)
    }
}

#[derive(Clone, Debug)]
pub enum BluetoothGATTDescriptor {
    #[cfg(feature = "native-bluetooth")]
    Btleplug(BtleplugGATTDescriptor),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothGATTDescriptor>),
}

impl BluetoothGATTDescriptor {
    #[cfg(feature = "bluetooth-test")]
    pub fn create_mock_descriptor(
        characteristic: BluetoothGATTCharacteristic,
        descriptor: String,
    ) -> Result<BluetoothGATTDescriptor, Box<dyn Error>> {
        #[allow(unreachable_patterns)]
        match characteristic {
            BluetoothGATTCharacteristic::Mock(fake_characteristic) => {
                Ok(BluetoothGATTDescriptor::Mock(
                    FakeBluetoothGATTDescriptor::new_empty(fake_characteristic, descriptor),
                ))
            },
            _ => Err(Box::from("The first parameter must be a mock structure")),
        }
    }

    pub fn get_id(&self) -> String {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_id(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_id(),
        }
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_uuid(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_uuid(),
        }
    }

    pub async fn read_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.read_value().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.read_value(),
        }
    }

    pub async fn write_value(&self, values: Vec<u8>) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.write_value(values).await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.write_value(values),
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_id(&self, id: String) {
        #[allow(irrefutable_let_patterns)]
        let Self::Mock(inner) = self else {
            return;
        };
        inner.set_id(id);
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_uuid(&self, uuid: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTDescriptor, set_uuid, uuid)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_value(&self, value: Vec<u8>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTDescriptor, set_value, Some(value))
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_flags(&self, flags: Vec<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTDescriptor, set_flags, flags)
    }
}
