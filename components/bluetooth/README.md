# Bluetooth

Servo-specific APIs to access Bluetooth devices.

Bluetooth related code is located in `bluetooth.rs`.

### Implementation

Underlying dependency crates:

- Android platform: [blurdroid](https://crates.io/crates/blurdroid)
- Linux platform: [blurz](https://crates.io/crates/blurz)
- macOS platform: [blurmac](https://crates.io/crates/blurmac)
- `Fake` prefixed structures: [blurmock](https://crates.io/crates/blurmock)

`Empty` prefixed structures are located in `empty.rs`.

### Usage

#### Without the *bluetooth-test* feature

There are three supported platforms (Android, Linux, macOS), on other platforms we fall back to a default (`Empty` prefixed) implementation. Each enum (`BluetoothAdapter`, `BluetoothDevice`, etc.) will contain only one variant for each targeted platform. See the following `BluetoothAdapter` example:

Android:

```rust
pub enum BluetoothAdapter {
    Android(Arc<BluetoothAdapterAndroid>),
}
```

Linux:

```rust
pub enum BluetoothAdapter {
    Bluez(Arc<BluetoothAdapterBluez>),
}
```

macOS:

```rust
pub enum BluetoothAdapter {
    Mac(Arc<BluetoothAdapterMac>),
}
```

Unsupported platforms:

```rust
pub enum BluetoothAdapter {
    Empty(Arc<BluetoothAdapterEmpty>),
}
```

You will have a platform specific adapter, e.g. on Android target, `BluetoothAdapter::init()` will create a `BluetoothAdapter::Android` enum variant, which wraps an `Arc<BluetoothAdapterAndroid>`.

```rust
pub fn init() -> Result<BluetoothAdapter, Box<Error>> {
    let blurdroid_adapter = try!(BluetoothAdapterAndroid::get_adapter());
    Ok(BluetoothAdapter::Android(Arc::new(blurdroid_adapter)))
}
```

On each platform you can call the same functions to reach the same GATT hierarchy elements. The following code can access the same Bluetooth device on all supported platforms:

```rust
use device::{BluetoothAdapter, BluetoothDevice};

fn main() {
    // Get the bluetooth adapter.
    let adapter = BluetoothAdpater::init().expect("No bluetooth adapter found!");
    // Get a device with the id 01:2A:00:4D:00:04 if it exists.
    let device = adapter.get_device("01:2A:00:4D:00:04".to_owned() /*device address*/)
                        .expect("No bluetooth device found!");
}
```

#### With the *bluetooth-test* feature

Each enum (`BluetoothAdapter`, `BluetoothDevice`, etc.) will contain one variant of the three possible default target, and a `Mock` variant, which wraps a `Fake` structure.

Android:

```rust
pub enum BluetoothAdapter {
    Android(Arc<BluetoothAdapterAndroid>),
    Mock(Arc<FakeBluetoothAdapter>),
}
```

Linux:

```rust
pub enum BluetoothAdapter {
    Bluez(Arc<BluetoothAdapterBluez>),
    Mock(Arc<FakeBluetoothAdapter>),
}
```

macOS:

```rust
pub enum BluetoothAdapter {
    Mac(Arc<BluetoothAdapterMac>),
    Mock(Arc<FakeBluetoothAdapter>),
}
```

Unsupported platforms:

```rust
pub enum BluetoothAdapter {
    Empty(Arc<BluetoothAdapterEmpty>),
    Mock(Arc<FakeBluetoothAdapter>),
}
```

Beside the platform specific structures, you can create and access mock adapters, devices, services etc. These mock structures implements all the platform specific functions too. To create a mock GATT hierarchy, first you need to call the `BluetoothAdapter::init_mock()` function, insted of `BluetoothAdapter::init()`.

```rust
use device::{BluetoothAdapter, BluetoothDevice};
use std::String;

// This function takes a BluetoothAdapter,
// and print the ids of the devices, which the adapter can find.
fn print_device_ids(adapter: &BluetoothAdpater) {
    let devices = match adapter.get_devices().expect("No devices on the adapter!");
    for device in devices {
        println!("{:?}", device.get_id());
    }
}

fn main() {
// This code uses a real adapter.
    // Get the bluetooth adapter.
    let adapter = BluetoothAdpater::init().expect("No bluetooth adapter found!");
    // Get a device with the id 01:2A:00:4D:00:04 if it exists.
    let device = adapter.get_device("01:2A:00:4D:00:04".to_owned() /*device address*/)
                        .expect("No bluetooth device found!");

// This code uses a mock adapter.
    // Creating a mock adapter.
    let mock_adapter = BluetoothAdpater::init_mock().unwrap();
    // Creating a mock device.
    let mock_device =
        BluetoothDevice::create_mock_device(mock_adapter,
                                            "device_id_string_goes_here".to_owned())
        .unwrap();
    // Changing its device_id.
    let new_device_id = String::from("new_device_id_string".to_owned());
    mock_device.set_id(new_device_id.clone());
    // Calling the get_id function must return the last id we set.
    assert_equals!(new_device_id, mock_device.get_id());
    // Getting the mock_device with its id
    // must return the same mock device object we created before.
    assert_equals!(Some(mock_device),
                   mock_adapter.get_device(new_device_id.clone()).unwrap());
    // The print_device_ids function accept real and mock adapters too.
    print_device_ids(&adapter);
    print_device_ids(&mock_adapter);
}
```

Calling a test function on a not `Mock` structure, will result an error with the message: `Error! Test functions are not supported on real devices!`.
