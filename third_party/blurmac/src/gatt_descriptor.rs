// Copyright (c) 2017 Akos Kiss.
//
// Licensed under the BSD 3-Clause License
// <LICENSE.md or https://opensource.org/licenses/BSD-3-Clause>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use std::error::Error;

use utils::NOT_SUPPORTED_ERROR;

#[derive(Clone, Debug)]
pub struct BluetoothGATTDescriptor {}

impl BluetoothGATTDescriptor {
    pub fn new(_descriptor: String) -> BluetoothGATTDescriptor {
        BluetoothGATTDescriptor {}
    }

    pub fn get_id(&self) -> String {
        String::new()
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn get_flags(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn read_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }

    pub fn write_value(&self, _values: Vec<u8>) -> Result<(), Box<dyn Error>> {
        Err(Box::from(NOT_SUPPORTED_ERROR))
    }
}
