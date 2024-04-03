/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use regex::Regex;

use crate::dom::bindings::codegen::UnionTypes::StringOrUnsignedLong;
use crate::dom::bindings::error::Error::Type;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;

pub type UUID = DOMString;
pub type BluetoothServiceUUID = StringOrUnsignedLong;
pub type BluetoothCharacteristicUUID = StringOrUnsignedLong;
pub type BluetoothDescriptorUUID = StringOrUnsignedLong;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothuuid
#[dom_struct]
pub struct BluetoothUUID {
    reflector_: Reflector,
}

//https://developer.bluetooth.org/gatt/services/Pages/ServicesHome.aspx
const BLUETOOTH_ASSIGNED_SERVICES: &[(&str, u32)] = &[
    ("org.bluetooth.service.alert_notification", 0x1811_u32),
    ("org.bluetooth.service.automation_io", 0x1815_u32),
    ("org.bluetooth.service.battery_service", 0x180f_u32),
    ("org.bluetooth.service.blood_pressure", 0x1810_u32),
    ("org.bluetooth.service.body_composition", 0x181b_u32),
    ("org.bluetooth.service.bond_management", 0x181e_u32),
    (
        "org.bluetooth.service.continuous_glucose_monitoring",
        0x181f_u32,
    ),
    ("org.bluetooth.service.current_time", 0x1805_u32),
    ("org.bluetooth.service.cycling_power", 0x1818_u32),
    (
        "org.bluetooth.service.cycling_speed_and_cadence",
        0x1816_u32,
    ),
    ("org.bluetooth.service.device_information", 0x180a_u32),
    ("org.bluetooth.service.environmental_sensing", 0x181a_u32),
    ("org.bluetooth.service.generic_access", 0x1800_u32),
    ("org.bluetooth.service.generic_attribute", 0x1801_u32),
    ("org.bluetooth.service.glucose", 0x1808_u32),
    ("org.bluetooth.service.health_thermometer", 0x1809_u32),
    ("org.bluetooth.service.heart_rate", 0x180d_u32),
    ("org.bluetooth.service.http_proxy", 0x1823_u32),
    ("org.bluetooth.service.human_interface_device", 0x1812_u32),
    ("org.bluetooth.service.immediate_alert", 0x1802_u32),
    ("org.bluetooth.service.indoor_positioning", 0x1821_u32),
    (
        "org.bluetooth.service.internet_protocol_support",
        0x1820_u32,
    ),
    ("org.bluetooth.service.link_loss", 0x1803_u32),
    ("org.bluetooth.service.location_and_navigation", 0x1819_u32),
    ("org.bluetooth.service.next_dst_change", 0x1807_u32),
    ("org.bluetooth.service.object_transfer", 0x1825_u32),
    ("org.bluetooth.service.phone_alert_status", 0x180e_u32),
    ("org.bluetooth.service.pulse_oximeter", 0x1822_u32),
    ("org.bluetooth.service.reference_time_update", 0x1806_u32),
    (
        "org.bluetooth.service.running_speed_and_cadence",
        0x1814_u32,
    ),
    ("org.bluetooth.service.scan_parameters", 0x1813_u32),
    ("org.bluetooth.service.transport_discovery", 0x1824),
    ("org.bluetooth.service.tx_power", 0x1804_u32),
    ("org.bluetooth.service.user_data", 0x181c_u32),
    ("org.bluetooth.service.weight_scale", 0x181d_u32),
];

//https://developer.bluetooth.org/gatt/services/Pages/ServicesHome.aspx
const BLUETOOTH_ASSIGNED_CHARCTERISTICS: &[(&str, u32)] = &[
    (
        "org.bluetooth.characteristic.aerobic_heart_rate_lower_limit",
        0x2a7e_u32,
    ),
    (
        "org.bluetooth.characteristic.aerobic_heart_rate_upper_limit",
        0x2a84_u32,
    ),
    ("org.bluetooth.characteristic.aerobic_threshold", 0x2a7f_u32),
    ("org.bluetooth.characteristic.age", 0x2a80_u32),
    ("org.bluetooth.characteristic.aggregate", 0x2a5a_u32),
    ("org.bluetooth.characteristic.alert_category_id", 0x2a43_u32),
    (
        "org.bluetooth.characteristic.alert_category_id_bit_mask",
        0x2a42_u32,
    ),
    ("org.bluetooth.characteristic.alert_level", 0x2a06_u32),
    (
        "org.bluetooth.characteristic.alert_notification_control_point",
        0x2a44_u32,
    ),
    ("org.bluetooth.characteristic.alert_status", 0x2a3f_u32),
    ("org.bluetooth.characteristic.altitude", 0x2ab3_u32),
    (
        "org.bluetooth.characteristic.anaerobic_heart_rate_lower_limit",
        0x2a81_u32,
    ),
    (
        "org.bluetooth.characteristic.anaerobic_heart_rate_upper_limit",
        0x2a82_u32,
    ),
    (
        "org.bluetooth.characteristic.anaerobic_threshold",
        0x2a83_u32,
    ),
    ("org.bluetooth.characteristic.analog", 0x2a58_u32),
    (
        "org.bluetooth.characteristic.apparent_wind_direction",
        0x2a73_u32,
    ),
    (
        "org.bluetooth.characteristic.apparent_wind_speed",
        0x2a72_u32,
    ),
    ("org.bluetooth.characteristic.gap.appearance", 0x2a01_u32),
    (
        "org.bluetooth.characteristic.barometric_pressure_trend",
        0x2aa3_u32,
    ),
    ("org.bluetooth.characteristic.battery_level", 0x2a19_u32),
    (
        "org.bluetooth.characteristic.blood_pressure_feature",
        0x2a49_u32,
    ),
    (
        "org.bluetooth.characteristic.blood_pressure_measurement",
        0x2a35_u32,
    ),
    (
        "org.bluetooth.characteristic.body_composition_feature",
        0x2a9b_u32,
    ),
    (
        "org.bluetooth.characteristic.body_composition_measurement",
        0x2a9c_u32,
    ),
    (
        "org.bluetooth.characteristic.body_sensor_location",
        0x2a38_u32,
    ),
    (
        "org.bluetooth.characteristic.bond_management_control_point",
        0x2aa4_u32,
    ),
    (
        "org.bluetooth.characteristic.bond_management_feature",
        0x2aa5_u32,
    ),
    (
        "org.bluetooth.characteristic.boot_keyboard_input_report",
        0x2a22_u32,
    ),
    (
        "org.bluetooth.characteristic.boot_keyboard_output_report",
        0x2a32_u32,
    ),
    (
        "org.bluetooth.characteristic.boot_mouse_input_report",
        0x2a33_u32,
    ),
    (
        "org.bluetooth.characteristic.gap.central_address_resolution_support",
        0x2aa6_u32,
    ),
    ("org.bluetooth.characteristic.cgm_feature", 0x2aa8_u32),
    ("org.bluetooth.characteristic.cgm_measurement", 0x2aa7_u32),
    (
        "org.bluetooth.characteristic.cgm_session_run_time",
        0x2aab_u32,
    ),
    (
        "org.bluetooth.characteristic.cgm_session_start_time",
        0x2aaa_u32,
    ),
    (
        "org.bluetooth.characteristic.cgm_specific_ops_control_point",
        0x2aac_u32,
    ),
    ("org.bluetooth.characteristic.cgm_status", 0x2aa9_u32),
    ("org.bluetooth.characteristic.csc_feature", 0x2a5c_u32),
    ("org.bluetooth.characteristic.csc_measurement", 0x2a5b_u32),
    ("org.bluetooth.characteristic.current_time", 0x2a2b_u32),
    (
        "org.bluetooth.characteristic.cycling_power_control_point",
        0x2a66_u32,
    ),
    (
        "org.bluetooth.characteristic.cycling_power_feature",
        0x2a65_u32,
    ),
    (
        "org.bluetooth.characteristic.cycling_power_measurement",
        0x2a63_u32,
    ),
    (
        "org.bluetooth.characteristic.cycling_power_vector",
        0x2a64_u32,
    ),
    (
        "org.bluetooth.characteristic.database_change_increment",
        0x2a99_u32,
    ),
    ("org.bluetooth.characteristic.date_of_birth", 0x2a85_u32),
    (
        "org.bluetooth.characteristic.date_of_threshold_assessment",
        0x2a86_u32,
    ),
    ("org.bluetooth.characteristic.date_time", 0x2a08_u32),
    ("org.bluetooth.characteristic.day_date_time", 0x2a0a_u32),
    ("org.bluetooth.characteristic.day_of_week", 0x2a09_u32),
    (
        "org.bluetooth.characteristic.descriptor_value_changed",
        0x2a7d_u32,
    ),
    ("org.bluetooth.characteristic.gap.device_name", 0x2a00_u32),
    ("org.bluetooth.characteristic.dew_point", 0x2a7b_u32),
    ("org.bluetooth.characteristic.digital", 0x2a56_u32),
    ("org.bluetooth.characteristic.dst_offset", 0x2a0d_u32),
    ("org.bluetooth.characteristic.elevation", 0x2a6c_u32),
    ("org.bluetooth.characteristic.email_address", 0x2a87_u32),
    ("org.bluetooth.characteristic.exact_time_256", 0x2a0c_u32),
    (
        "org.bluetooth.characteristic.fat_burn_heart_rate_lower_limit",
        0x2a88_u32,
    ),
    (
        "org.bluetooth.characteristic.fat_burn_heart_rate_upper_limit",
        0x2a89_u32,
    ),
    (
        "org.bluetooth.characteristic.firmware_revision_string",
        0x2a26_u32,
    ),
    ("org.bluetooth.characteristic.first_name", 0x2a8a_u32),
    (
        "org.bluetooth.characteristic.five_zone_heart_rate_limits",
        0x2a8b_u32,
    ),
    ("org.bluetooth.characteristic.floor_number", 0x2ab2_u32),
    ("org.bluetooth.characteristic.gender", 0x2a8c_u32),
    ("org.bluetooth.characteristic.glucose_feature", 0x2a51_u32),
    (
        "org.bluetooth.characteristic.glucose_measurement",
        0x2a18_u32,
    ),
    (
        "org.bluetooth.characteristic.glucose_measurement_context",
        0x2a34_u32,
    ),
    ("org.bluetooth.characteristic.gust_factor", 0x2a74_u32),
    (
        "org.bluetooth.characteristic.hardware_revision_string",
        0x2a27_u32,
    ),
    (
        "org.bluetooth.characteristic.heart_rate_control_point",
        0x2a39_u32,
    ),
    ("org.bluetooth.characteristic.heart_rate_max", 0x2a8d_u32),
    (
        "org.bluetooth.characteristic.heart_rate_measurement",
        0x2a37_u32,
    ),
    ("org.bluetooth.characteristic.heat_index", 0x2a7a_u32),
    ("org.bluetooth.characteristic.height", 0x2a8e_u32),
    ("org.bluetooth.characteristic.hid_control_point", 0x2a4c_u32),
    ("org.bluetooth.characteristic.hid_information", 0x2a4a_u32),
    ("org.bluetooth.characteristic.hip_circumference", 0x2a8f_u32),
    (
        "org.bluetooth.characteristic.http_control_point",
        0x2aba_u32,
    ),
    ("org.bluetooth.characteristic.http_entity_body", 0x2ab9_u32),
    ("org.bluetooth.characteristic.http_headers", 0x2ab7_u32),
    ("org.bluetooth.characteristic.http_status_code", 0x2ab8_u32),
    ("org.bluetooth.characteristic.https_security", 0x2abb_u32),
    ("org.bluetooth.characteristic.humidity", 0x2a6f_u32),
    (
        "org.bluetooth.characteristic.ieee_11073-20601_regulatory_certification_data_list",
        0x2a2a_u32,
    ),
    (
        "org.bluetooth.characteristic.indoor_positioning_configuration",
        0x2aad_u32,
    ),
    (
        "org.bluetooth.characteristic.intermediate_cuff_pressure",
        0x2a36_u32,
    ),
    (
        "org.bluetooth.characteristic.intermediate_temperature",
        0x2a1e_u32,
    ),
    ("org.bluetooth.characteristic.irradiance", 0x2a77_u32),
    ("org.bluetooth.characteristic.language", 0x2aa2_u32),
    ("org.bluetooth.characteristic.last_name", 0x2a90_u32),
    ("org.bluetooth.characteristic.latitude", 0x2aae_u32),
    ("org.bluetooth.characteristic.ln_control_point", 0x2a6b_u32),
    ("org.bluetooth.characteristic.ln_feature", 0x2a6a_u32),
    (
        "org.bluetooth.characteristic.local_east_coordinate.xml",
        0x2ab1_u32,
    ),
    (
        "org.bluetooth.characteristic.local_north_coordinate",
        0x2ab0_u32,
    ),
    (
        "org.bluetooth.characteristic.local_time_information",
        0x2a0f_u32,
    ),
    (
        "org.bluetooth.characteristic.location_and_speed",
        0x2a67_u32,
    ),
    ("org.bluetooth.characteristic.location_name", 0x2ab5_u32),
    ("org.bluetooth.characteristic.longitude", 0x2aaf_u32),
    (
        "org.bluetooth.characteristic.magnetic_declination",
        0x2a2c_u32,
    ),
    (
        "org.bluetooth.characteristic.magnetic_flux_density_2d",
        0x2aa0_u32,
    ),
    (
        "org.bluetooth.characteristic.magnetic_flux_density_3d",
        0x2aa1_u32,
    ),
    (
        "org.bluetooth.characteristic.manufacturer_name_string",
        0x2a29_u32,
    ),
    (
        "org.bluetooth.characteristic.maximum_recommended_heart_rate",
        0x2a91_u32,
    ),
    (
        "org.bluetooth.characteristic.measurement_interval",
        0x2a21_u32,
    ),
    (
        "org.bluetooth.characteristic.model_number_string",
        0x2a24_u32,
    ),
    ("org.bluetooth.characteristic.navigation", 0x2a68_u32),
    ("org.bluetooth.characteristic.new_alert", 0x2a46_u32),
    (
        "org.bluetooth.characteristic.object_action_control_point",
        0x2ac5_u32,
    ),
    ("org.bluetooth.characteristic.object_changed", 0x2ac8_u32),
    (
        "org.bluetooth.characteristic.object_first_created",
        0x2ac1_u32,
    ),
    ("org.bluetooth.characteristic.object_id", 0x2ac3_u32),
    (
        "org.bluetooth.characteristic.object_last_modified",
        0x2ac2_u32,
    ),
    (
        "org.bluetooth.characteristic.object_list_control_point",
        0x2ac6_u32,
    ),
    (
        "org.bluetooth.characteristic.object_list_filter",
        0x2ac7_u32,
    ),
    ("org.bluetooth.characteristic.object_name", 0x2abe_u32),
    ("org.bluetooth.characteristic.object_properties", 0x2ac4_u32),
    ("org.bluetooth.characteristic.object_size", 0x2ac0_u32),
    ("org.bluetooth.characteristic.object_type", 0x2abf_u32),
    ("org.bluetooth.characteristic.ots_feature", 0x2abd_u32),
    (
        "org.bluetooth.characteristic.gap.peripheral_preferred_connection_parameters",
        0x2a04_u32,
    ),
    (
        "org.bluetooth.characteristic.gap.peripheral_privacy_flag",
        0x2a02_u32,
    ),
    (
        "org.bluetooth.characteristic.plx_continuous_measurement",
        0x2a5f_u32,
    ),
    ("org.bluetooth.characteristic.plx_features", 0x2a60_u32),
    (
        "org.bluetooth.characteristic.plx_spot_check_measurement",
        0x2a5e_u32,
    ),
    ("org.bluetooth.characteristic.pnp_id", 0x2a50_u32),
    (
        "org.bluetooth.characteristic.pollen_concentration",
        0x2a75_u32,
    ),
    ("org.bluetooth.characteristic.position_quality", 0x2a69_u32),
    ("org.bluetooth.characteristic.pressure", 0x2a6d_u32),
    ("org.bluetooth.characteristic.protocol_mode", 0x2a4e_u32),
    ("org.bluetooth.characteristic.rainfall", 0x2a78_u32),
    (
        "org.bluetooth.characteristic.gap.reconnection_address",
        0x2a03_u32,
    ),
    (
        "org.bluetooth.characteristic.record_access_control_point",
        0x2a52_u32,
    ),
    (
        "org.bluetooth.characteristic.reference_time_information",
        0x2a14_u32,
    ),
    ("org.bluetooth.characteristic.report", 0x2a4d_u32),
    ("org.bluetooth.characteristic.report_map", 0x2a4b_u32),
    (
        "org.bluetooth.characteristic.resting_heart_rate",
        0x2a92_u32,
    ),
    (
        "org.bluetooth.characteristic.ringer_control_point",
        0x2a40_u32,
    ),
    ("org.bluetooth.characteristic.ringer_setting", 0x2a41_u32),
    ("org.bluetooth.characteristic.rsc_feature", 0x2a54_u32),
    ("org.bluetooth.characteristic.rsc_measurement", 0x2a53_u32),
    ("org.bluetooth.characteristic.sc_control_point", 0x2a55_u32),
    (
        "org.bluetooth.characteristic.scan_interval_window",
        0x2a4f_u32,
    ),
    ("org.bluetooth.characteristic.scan_refresh", 0x2a31_u32),
    ("org.bluetooth.characteristic.sensor_location", 0x2a5d_u32),
    (
        "org.bluetooth.characteristic.serial_number_string",
        0x2a25_u32,
    ),
    (
        "org.bluetooth.characteristic.gatt.service_changed",
        0x2a05_u32,
    ),
    (
        "org.bluetooth.characteristic.software_revision_string",
        0x2a28_u32,
    ),
    (
        "org.bluetooth.characteristic.sport_type_for_aerobic_and_anaerobic_thresholds",
        0x2a93_u32,
    ),
    (
        "org.bluetooth.characteristic.supported_new_alert_category",
        0x2a47_u32,
    ),
    (
        "org.bluetooth.characteristic.supported_unread_alert_category",
        0x2a48_u32,
    ),
    ("org.bluetooth.characteristic.system_id", 0x2a23_u32),
    ("org.bluetooth.characteristic.tds_control_point", 0x2abc_u32),
    ("org.bluetooth.characteristic.temperature", 0x2a6e_u32),
    (
        "org.bluetooth.characteristic.temperature_measurement",
        0x2a1c_u32,
    ),
    ("org.bluetooth.characteristic.temperature_type", 0x2a1d_u32),
    (
        "org.bluetooth.characteristic.three_zone_heart_rate_limits",
        0x2a94_u32,
    ),
    ("org.bluetooth.characteristic.time_accuracy", 0x2a12_u32),
    ("org.bluetooth.characteristic.time_source", 0x2a13_u32),
    (
        "org.bluetooth.characteristic.time_update_control_point",
        0x2a16_u32,
    ),
    ("org.bluetooth.characteristic.time_update_state", 0x2a17_u32),
    ("org.bluetooth.characteristic.time_with_dst", 0x2a11_u32),
    ("org.bluetooth.characteristic.time_zone", 0x2a0e_u32),
    (
        "org.bluetooth.characteristic.true_wind_direction",
        0x2a71_u32,
    ),
    ("org.bluetooth.characteristic.true_wind_speed", 0x2a70_u32),
    (
        "org.bluetooth.characteristic.two_zone_heart_rate_limit",
        0x2a95_u32,
    ),
    ("org.bluetooth.characteristic.tx_power_level", 0x2a07_u32),
    ("org.bluetooth.characteristic.uncertainty", 0x2ab4_u32),
    (
        "org.bluetooth.characteristic.unread_alert_status",
        0x2a45_u32,
    ),
    ("org.bluetooth.characteristic.uri", 0x2ab6_u32),
    (
        "org.bluetooth.characteristic.user_control_point",
        0x2a9f_u32,
    ),
    ("org.bluetooth.characteristic.user_index", 0x2a9a_u32),
    ("org.bluetooth.characteristic.uv_index", 0x2a76_u32),
    ("org.bluetooth.characteristic.vo2_max", 0x2a96_u32),
    (
        "org.bluetooth.characteristic.waist_circumference",
        0x2a97_u32,
    ),
    ("org.bluetooth.characteristic.weight", 0x2a98_u32),
    (
        "org.bluetooth.characteristic.weight_measurement",
        0x2a9d_u32,
    ),
    (
        "org.bluetooth.characteristic.weight_scale_feature",
        0x2a9e_u32,
    ),
    ("org.bluetooth.characteristic.wind_chill", 0x2a79_u32),
];

//https://developer.bluetooth.org/gatt/services/Pages/ServicesHome.aspx
const BLUETOOTH_ASSIGNED_DESCRIPTORS: &[(&str, u32)] = &[
    (
        "org.bluetooth.descriptor.gatt.characteristic_extended_properties",
        0x2900_u32,
    ),
    (
        "org.bluetooth.descriptor.gatt.characteristic_user_description",
        0x2901_u32,
    ),
    (
        "org.bluetooth.descriptor.gatt.client_characteristic_configuration",
        0x2902_u32,
    ),
    (
        "org.bluetooth.descriptor.gatt.server_characteristic_configuration",
        0x2903_u32,
    ),
    (
        "org.bluetooth.descriptor.gatt.characteristic_presentation_format",
        0x2904_u32,
    ),
    (
        "org.bluetooth.descriptor.gatt.characteristic_aggregate_format",
        0x2905_u32,
    ),
    ("org.bluetooth.descriptor.valid_range", 0x2906_u32),
    (
        "org.bluetooth.descriptor.external_report_reference",
        0x2907_u32,
    ),
    ("org.bluetooth.descriptor.report_reference", 0x2908_u32),
    ("org.bluetooth.descriptor.number_of_digitals", 0x2909_u32),
    ("org.bluetooth.descriptor.value_trigger_setting", 0x290a_u32),
    ("org.bluetooth.descriptor.es_configuration", 0x290b_u32),
    ("org.bluetooth.descriptor.es_measurement", 0x290c_u32),
    ("org.bluetooth.descriptor.es_trigger_setting", 0x290d_u32),
    ("org.bluetooth.descriptor.time_trigger_setting", 0x290e_u32),
];

const BASE_UUID: &str = "-0000-1000-8000-00805f9b34fb";
const SERVICE_PREFIX: &str = "org.bluetooth.service";
const CHARACTERISTIC_PREFIX: &str = "org.bluetooth.characteristic";
const DESCRIPTOR_PREFIX: &str = "org.bluetooth.descriptor";
const VALID_UUID_REGEX: &str = "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";
// https://cs.chromium.org/chromium/src/third_party/WebKit/Source/modules/bluetooth/BluetoothUUID.cpp?l=314
const UUID_ERROR_MESSAGE: &str = "It must be a valid UUID alias (e.g. 0x1234), \
    UUID (lowercase hex characters e.g. '00001234-0000-1000-8000-00805f9b34fb'),\nor recognized standard name from";
// https://cs.chromium.org/chromium/src/third_party/WebKit/Source/modules/bluetooth/BluetoothUUID.cpp?l=321
const SERVICES_ERROR_MESSAGE: &str =
    "https://developer.bluetooth.org/gatt/services/Pages/ServicesHome.aspx\
     \ne.g. 'alert_notification'.";
// https://cs.chromium.org/chromium/src/third_party/WebKit/Source/modules/bluetooth/BluetoothUUID.cpp?l=327
const CHARACTERISTIC_ERROR_MESSAGE: &str =
    "https://developer.bluetooth.org/gatt/characteristics/Pages/\
     CharacteristicsHome.aspx\ne.g. 'aerobic_heart_rate_lower_limit'.";
// https://cs.chromium.org/chromium/src/third_party/WebKit/Source/modules/bluetooth/BluetoothUUID.cpp?l=333
const DESCRIPTOR_ERROR_MESSAGE: &str = "https://developer.bluetooth.org/gatt/descriptors/Pages/\
     DescriptorsHomePage.aspx\ne.g. 'gatt.characteristic_presentation_format'.";

#[allow(non_snake_case)]
impl BluetoothUUID {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothuuid-canonicaluuid
    pub fn CanonicalUUID(_: &Window, alias: u32) -> UUID {
        canonical_uuid(alias)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothuuid-getservice
    pub fn GetService(_: &Window, name: BluetoothServiceUUID) -> Fallible<UUID> {
        Self::service(name)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothuuid-getcharacteristic
    pub fn GetCharacteristic(_: &Window, name: BluetoothCharacteristicUUID) -> Fallible<UUID> {
        Self::characteristic(name)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothuuid-getdescriptor
    pub fn GetDescriptor(_: &Window, name: BluetoothDescriptorUUID) -> Fallible<UUID> {
        Self::descriptor(name)
    }
}

impl BluetoothUUID {
    pub fn service(name: BluetoothServiceUUID) -> Fallible<UUID> {
        resolve_uuid_name(name, BLUETOOTH_ASSIGNED_SERVICES, SERVICE_PREFIX)
    }

    pub fn characteristic(name: BluetoothCharacteristicUUID) -> Fallible<UUID> {
        resolve_uuid_name(
            name,
            BLUETOOTH_ASSIGNED_CHARCTERISTICS,
            CHARACTERISTIC_PREFIX,
        )
    }

    pub fn descriptor(name: BluetoothDescriptorUUID) -> Fallible<UUID> {
        resolve_uuid_name(name, BLUETOOTH_ASSIGNED_DESCRIPTORS, DESCRIPTOR_PREFIX)
    }
}

impl Clone for StringOrUnsignedLong {
    fn clone(&self) -> StringOrUnsignedLong {
        match self {
            StringOrUnsignedLong::String(s) => StringOrUnsignedLong::String(s.clone()),
            &StringOrUnsignedLong::UnsignedLong(ul) => StringOrUnsignedLong::UnsignedLong(ul),
        }
    }
}

fn canonical_uuid(alias: u32) -> UUID {
    UUID::from(format!("{:08x}", &alias) + BASE_UUID)
}

// https://webbluetoothcg.github.io/web-bluetooth/#resolveuuidname
fn resolve_uuid_name(
    name: StringOrUnsignedLong,
    assigned_numbers_table: &'static [(&'static str, u32)],
    prefix: &str,
) -> Fallible<DOMString> {
    match name {
        // Step 1.
        StringOrUnsignedLong::UnsignedLong(unsigned32) => Ok(canonical_uuid(unsigned32)),
        StringOrUnsignedLong::String(dstring) => {
            // Step 2.
            let regex = Regex::new(VALID_UUID_REGEX).unwrap();
            if regex.is_match(&dstring) {
                Ok(dstring)
            } else {
                // Step 3.
                let concatenated = format!("{}.{}", prefix, dstring);
                let is_in_table = assigned_numbers_table.iter().find(|p| p.0 == concatenated);
                match is_in_table {
                    Some(&(_, alias)) => Ok(canonical_uuid(alias)),
                    None => {
                        let (attribute_type, error_url_message) = match prefix {
                            SERVICE_PREFIX => ("Service", SERVICES_ERROR_MESSAGE),
                            CHARACTERISTIC_PREFIX => {
                                ("Characteristic", CHARACTERISTIC_ERROR_MESSAGE)
                            },
                            DESCRIPTOR_PREFIX => ("Descriptor", DESCRIPTOR_ERROR_MESSAGE),
                            _ => unreachable!(),
                        };
                        // Step 4.
                        Err(Type(format!(
                            "Invalid {} name : '{}'.\n{} {}",
                            attribute_type, dstring, UUID_ERROR_MESSAGE, error_url_message
                        )))
                    },
                }
            }
        },
    }
}
