/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothDeviceBinding;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::{BluetoothDeviceMethods, VendorIDSource};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root, MutHeap, MutNullableHeap};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bluetoothadvertisingdata::BluetoothAdvertisingData;
use dom::bluetoothremotegattserver::BluetoothRemoteGATTServer;
use util::str::DOMString;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdevice
#[dom_struct]
pub struct BluetoothDevice {
    reflector_: Reflector,
    id: DOMString,
    name: Option<DOMString>,
    adData: MutHeap<JS<BluetoothAdvertisingData>>,
    deviceClass: Option<u32>,
    vendorIDSource: Option<VendorIDSource>,
    vendorID: Option<u32>,
    productID: Option<u32>,
    productVersion: Option<u32>,
    gatt: MutNullableHeap<JS<BluetoothRemoteGATTServer>>,
}

impl BluetoothDevice {
    pub fn new_inherited(id: DOMString,
                         name: Option<DOMString>,
                         adData: &BluetoothAdvertisingData,
                         deviceClass: Option<u32>,
                         vendorIDSource: Option<VendorIDSource>,
                         vendorID: Option<u32>,
                         productID: Option<u32>,
                         productVersion: Option<u32>)
                         -> BluetoothDevice {
        BluetoothDevice {
            reflector_: Reflector::new(),
            id: id,
            name: name,
            adData: MutHeap::new(adData),
            deviceClass: deviceClass,
            vendorIDSource: vendorIDSource,
            vendorID: vendorID,
            productID: productID,
            productVersion: productVersion,
            gatt: Default::default(),
        }
    }

    pub fn new(global: GlobalRef,
               id: DOMString,
               name: Option<DOMString>,
               adData: &BluetoothAdvertisingData,
               deviceClass: Option<u32>,
               vendorIDSource: Option<VendorIDSource>,
               vendorID: Option<u32>,
               productID: Option<u32>,
               productVersion: Option<u32>)
               -> Root<BluetoothDevice> {
        reflect_dom_object(box BluetoothDevice::new_inherited(id,
                                                              name,
                                                              adData,
                                                              deviceClass,
                                                              vendorIDSource,
                                                              vendorID,
                                                              productID,
                                                              productVersion),
                           global,
                           BluetoothDeviceBinding::Wrap)
    }
}

impl BluetoothDeviceMethods for BluetoothDevice {

     // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-id
    fn Id(&self) -> DOMString {
        self.id.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-name
    fn GetName(&self) -> Option<DOMString> {
        self.name.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-addata
    fn AdData(&self) -> Root<BluetoothAdvertisingData> {
        self.adData.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-deviceclass
    fn GetDeviceClass(&self) -> Option<u32> {
        self.deviceClass
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-vendoridsource
    fn GetVendorIDSource(&self) -> Option<VendorIDSource> {
        self.vendorIDSource
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-vendorid
    fn GetVendorID(&self) -> Option<u32> {
        self.vendorID
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-productid
    fn GetProductID(&self) -> Option<u32> {
        self.productID
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-productversion
    fn GetProductVersion(&self) -> Option<u32> {
        self.productVersion
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-gatt
    fn Gatt(&self) -> Root<BluetoothRemoteGATTServer> {
        self.gatt.or_init(|| BluetoothRemoteGATTServer::new(self.global().r(), self))
    }
}
