use openxr::{
    sys::{
        BD_CONTROLLER_INTERACTION_EXTENSION_NAME, EXT_HAND_INTERACTION_EXTENSION_NAME,
        EXT_HP_MIXED_REALITY_CONTROLLER_EXTENSION_NAME,
        EXT_SAMSUNG_ODYSSEY_CONTROLLER_EXTENSION_NAME, FB_HAND_TRACKING_AIM_EXTENSION_NAME,
        FB_TOUCH_CONTROLLER_PRO_EXTENSION_NAME,
        HTC_VIVE_COSMOS_CONTROLLER_INTERACTION_EXTENSION_NAME,
        HTC_VIVE_FOCUS3_CONTROLLER_INTERACTION_EXTENSION_NAME,
        META_TOUCH_CONTROLLER_PLUS_EXTENSION_NAME, ML_ML2_CONTROLLER_INTERACTION_EXTENSION_NAME,
    },
    ExtensionSet,
};

#[macro_export]
macro_rules! ext_string {
    ($ext_name:expr) => {
        std::str::from_utf8($ext_name).unwrap()
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractionProfileType {
    KhrSimpleController,
    BytedancePicoNeo3Controller,
    BytedancePico4Controller,
    BytedancePicoG3Controller,
    GoogleDaydreamController,
    HpMixedRealityController,
    HtcViveController,
    HtcViveCosmosController,
    HtcViveFocus3Controller,
    MagicLeap2Controller,
    MicrosoftMixedRealityMotionController,
    OculusGoController,
    OculusTouchController,
    FacebookTouchControllerPro,
    MetaTouchPlusController,
    MetaTouchControllerRiftCv1,
    MetaTouchControllerQuest1RiftS,
    MetaTouchControllerQuest2,
    SamsungOdysseyController,
    ValveIndexController,
    ExtHandInteraction,
    FbHandTrackingAim,
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionProfile<'a> {
    pub profile_type: InteractionProfileType,
    /// The interaction profile path
    pub path: &'static str,
    /// The OpenXR extension, if any, required to use this profile
    pub required_extension: Option<&'a [u8]>,
    /// Trigger, Grip, Touchpad, Thumbstick
    pub standard_buttons: &'a [&'a str],
    /// Touchpad X, Touchpad Y, Thumbstick X, Thumbstick Y
    pub standard_axes: &'a [&'a str],
    /// Any additional buttons on the left controller
    pub left_buttons: &'a [&'a str],
    /// Any additional buttons on the right controller
    pub right_buttons: &'a [&'a str],
    /// The corresponding WebXR Input Profile names
    pub profiles: &'a [&'a str],
}

pub static KHR_SIMPLE_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::KhrSimpleController,
    path: "/interaction_profiles/khr/simple_controller",
    required_extension: None,
    standard_buttons: &["select/click", "", "", ""],
    standard_axes: &["", "", "", ""],
    left_buttons: &[],
    right_buttons: &[],
    profiles: &["generic-trigger"],
};

pub static BYTEDANCE_PICO_NEO3_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::BytedancePicoNeo3Controller,
    path: "/interaction_profiles/bytedance/pico_neo3_controller",
    required_extension: Some(BD_CONTROLLER_INTERACTION_EXTENSION_NAME),
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &["pico-neo3", "generic-trigger-squeeze-thumbstick"],
};

pub static BYTEDANCE_PICO_4_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::BytedancePico4Controller,
    path: "/interaction_profiles/bytedance/pico4_controller",
    required_extension: Some(BD_CONTROLLER_INTERACTION_EXTENSION_NAME),
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &["pico-4", "generic-trigger-squeeze-thumbstick"],
};

pub static BYTEDANCE_PICO_G3_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::BytedancePicoG3Controller,
    path: "/interaction_profiles/bytedance/pico_g3_controller",
    required_extension: Some(BD_CONTROLLER_INTERACTION_EXTENSION_NAME),
    standard_buttons: &["trigger/value", "", "", "thumbstick/click"],
    // Note: X/Y components not listed in the OpenXR spec currently due to vendor error.
    // See <https://github.com/KhronosGroup/OpenXR-Docs/issues/158>
    // It also uses the thumbstick path despite clearly being a touchpad, so
    // move those values into the touchpad axes slots
    standard_axes: &["thumbstick/x", "thumbstick/y", "", ""],
    left_buttons: &[],
    right_buttons: &[],
    // Note: There is no corresponding WebXR Input profile for the Pico G3,
    // but the controller seems identical to the G2, so use that instead.
    profiles: &["pico-g2", "generic-trigger-touchpad"],
};

pub static GOOGLE_DAYDREAM_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::GoogleDaydreamController,
    path: "/interaction_profiles/google/daydream_controller",
    required_extension: None,
    standard_buttons: &["select/click", "", "trackpad/click", ""],
    standard_axes: &["trackpad/x", "trackpad/y", "", ""],
    left_buttons: &[],
    right_buttons: &[],
    profiles: &["google-daydream", "generic-touchpad"],
};

pub static HP_MIXED_REALITY_MOTION_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::HpMixedRealityController,
    path: "/interaction_profiles/hp/mixed_reality_controller",
    required_extension: Some(EXT_HP_MIXED_REALITY_CONTROLLER_EXTENSION_NAME),
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &[
        "hp-mixed-reality",
        "oculus-touch",
        "generic-trigger-squeeze-thumbstick",
    ],
};

pub static HTC_VIVE_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::HtcViveController,
    path: "/interaction_profiles/htc/vive_controller",
    required_extension: None,
    standard_buttons: &["trigger/value", "squeeze/click", "trackpad/click", ""],
    standard_axes: &["trackpad/x", "trackpad/y", "", ""],
    left_buttons: &[],
    right_buttons: &[],
    profiles: &["htc-vive", "generic-trigger-squeeze-touchpad"],
};

pub static HTC_VIVE_COSMOS_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::HtcViveCosmosController,
    path: "/interaction_profiles/htc/vive_cosmos_controller",
    required_extension: Some(HTC_VIVE_COSMOS_CONTROLLER_INTERACTION_EXTENSION_NAME),
    standard_buttons: &["trigger/value", "squeeze/click", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &["htc-vive-cosmos", "generic-trigger-squeeze-thumbstick"],
};

pub static HTC_VIVE_FOCUS3_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::HtcViveFocus3Controller,
    path: "/interaction_profiles/htc/vive_focus3_controller",
    required_extension: Some(HTC_VIVE_FOCUS3_CONTROLLER_INTERACTION_EXTENSION_NAME),
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &["htc-vive-focus-3", "generic-trigger-squeeze-thumbstick"],
};

pub static MAGIC_LEAP_2_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::MagicLeap2Controller,
    path: "/interaction_profiles/ml/ml2_controller",
    required_extension: Some(ML_ML2_CONTROLLER_INTERACTION_EXTENSION_NAME),
    standard_buttons: &["trigger/value", "", "trackpad/click", ""],
    standard_axes: &["trackpad/x", "trackpad/y", "", ""],
    left_buttons: &[],
    right_buttons: &[],
    // Note: There is no corresponding WebXR Input profile for the Magic Leap 2,
    // but the controller seems mostly identical to the 1, so use that instead.
    profiles: &["magicleap-one", "generic-trigger-squeeze-touchpad"],
};

pub static MICROSOFT_MIXED_REALITY_MOTION_CONTROLLER_PROFILE: InteractionProfile =
    InteractionProfile {
        profile_type: InteractionProfileType::MicrosoftMixedRealityMotionController,
        path: "/interaction_profiles/microsoft/motion_controller",
        required_extension: None,
        standard_buttons: &[
            "trigger/value",
            "squeeze/click",
            "trackpad/click",
            "thumbstick/click",
        ],
        standard_axes: &["trackpad/x", "trackpad/y", "thumbstick/x", "thumbstick/y"],
        left_buttons: &[],
        right_buttons: &[],
        profiles: &[
            "microsoft-mixed-reality",
            "generic-trigger-squeeze-touchpad-thumbstick",
        ],
    };

pub static OCULUS_GO_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::OculusGoController,
    path: "/interaction_profiles/oculus/go_controller",
    required_extension: None,
    standard_buttons: &["trigger/click", "", "trackpad/click", ""],
    standard_axes: &["trackpad/x", "trackpad/y", "", ""],
    left_buttons: &[],
    right_buttons: &[],
    profiles: &["oculus-go", "generic-trigger-touchpad"],
};

pub static OCULUS_TOUCH_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::OculusTouchController,
    path: "/interaction_profiles/oculus/touch_controller",
    required_extension: None,
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &[
        "oculus-touch-v3",
        "oculus-touch-v2",
        "oculus-touch",
        "generic-trigger-squeeze-thumbstick",
    ],
};

pub static FACEBOOK_TOUCH_CONTROLLER_PRO_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::FacebookTouchControllerPro,
    path: "/interaction_profiles/facebook/touch_controller_pro",
    required_extension: Some(FB_TOUCH_CONTROLLER_PRO_EXTENSION_NAME),
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &[
        "meta-quest-touch-pro",
        "oculus-touch-v2",
        "oculus-touch",
        "generic-trigger-squeeze-thumbstick",
    ],
};

pub static META_TOUCH_CONTROLLER_PLUS_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::MetaTouchPlusController,
    path: "/interaction_profiles/meta/touch_controller_plus",
    required_extension: Some(META_TOUCH_CONTROLLER_PLUS_EXTENSION_NAME),
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &[
        "meta-quest-touch-plus",
        "oculus-touch-v3",
        "oculus-touch",
        "generic-trigger-squeeze-thumbstick",
    ],
};

pub static META_TOUCH_CONTROLLER_RIFT_CV1_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::MetaTouchControllerRiftCv1,
    path: "/interaction_profiles/meta/touch_controller_rift_cv1",
    required_extension: None,
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &["oculus-touch", "generic-trigger-squeeze-thumbstick"],
};

pub static META_TOUCH_CONTROLLER_QUEST_1_RIFT_S_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::MetaTouchControllerQuest1RiftS,
    path: "/interaction_profiles/meta/touch_controller_quest_1_rift_s",
    required_extension: None,
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &[
        "oculus-touch-v2",
        "oculus-touch",
        "generic-trigger-squeeze-thumbstick",
    ],
};

pub static META_TOUCH_CONTROLLER_QUEST_2_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::MetaTouchControllerQuest2,
    path: "/interaction_profiles/meta/touch_controller_quest_2",
    required_extension: None,
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["", "", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["x/click", "y/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &[
        "oculus-touch-v3",
        "oculus-touch-v2",
        "oculus-touch",
        "generic-trigger-squeeze-thumbstick",
    ],
};

pub static SAMSUNG_ODYSSEY_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::SamsungOdysseyController,
    path: "/interaction_profiles/samsung/odyssey_controller",
    required_extension: Some(EXT_SAMSUNG_ODYSSEY_CONTROLLER_EXTENSION_NAME),
    standard_buttons: &[
        "trigger/value",
        "squeeze/click",
        "trackpad/click",
        "thumbstick/click",
    ],
    standard_axes: &["trackpad/x", "trackpad/y", "thumbstick/x", "thumbstick/y"],
    left_buttons: &[],
    right_buttons: &[],
    profiles: &[
        "samsung-odyssey",
        "microsoft-mixed-reality",
        "generic-trigger-squeeze-touchpad-thumbstick",
    ],
};

pub static VALVE_INDEX_CONTROLLER_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::ValveIndexController,
    path: "/interaction_profiles/valve/index_controller",
    required_extension: None,
    standard_buttons: &["trigger/value", "squeeze/value", "", "thumbstick/click"],
    standard_axes: &["trackpad/x", "trackpad/y", "thumbstick/x", "thumbstick/y"],
    left_buttons: &["a/click", "b/click"],
    right_buttons: &["a/click", "b/click"],
    profiles: &["valve-index", "generic-trigger-squeeze-touchpad-thumbstick"],
};

pub static EXT_HAND_INTERACTION_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::ExtHandInteraction,
    path: "/interaction_profiles/ext/hand_interaction_ext",
    required_extension: Some(EXT_HAND_INTERACTION_EXTENSION_NAME),
    standard_buttons: &["pinch_ext/value", "", "", ""],
    standard_axes: &["", "", "", ""],
    left_buttons: &[],
    right_buttons: &[],
    profiles: &["generic-hand-select", "generic-hand"],
};

pub static FB_HAND_TRACKING_AIM_PROFILE: InteractionProfile = InteractionProfile {
    profile_type: InteractionProfileType::FbHandTrackingAim,
    path: "",
    required_extension: Some(FB_HAND_TRACKING_AIM_EXTENSION_NAME),
    standard_buttons: &["", "", "", ""],
    standard_axes: &["", "", "", ""],
    left_buttons: &[],
    right_buttons: &[],
    profiles: &["generic-hand-select", "generic-hand"],
};

pub static INTERACTION_PROFILES: [InteractionProfile; 22] = [
    KHR_SIMPLE_CONTROLLER_PROFILE,
    BYTEDANCE_PICO_NEO3_CONTROLLER_PROFILE,
    BYTEDANCE_PICO_4_CONTROLLER_PROFILE,
    BYTEDANCE_PICO_G3_CONTROLLER_PROFILE,
    GOOGLE_DAYDREAM_CONTROLLER_PROFILE,
    HP_MIXED_REALITY_MOTION_CONTROLLER_PROFILE,
    HTC_VIVE_CONTROLLER_PROFILE,
    HTC_VIVE_COSMOS_CONTROLLER_PROFILE,
    HTC_VIVE_FOCUS3_CONTROLLER_PROFILE,
    MAGIC_LEAP_2_CONTROLLER_PROFILE,
    MICROSOFT_MIXED_REALITY_MOTION_CONTROLLER_PROFILE,
    OCULUS_GO_CONTROLLER_PROFILE,
    OCULUS_TOUCH_CONTROLLER_PROFILE,
    FACEBOOK_TOUCH_CONTROLLER_PRO_PROFILE,
    META_TOUCH_CONTROLLER_PLUS_PROFILE,
    META_TOUCH_CONTROLLER_RIFT_CV1_PROFILE,
    META_TOUCH_CONTROLLER_QUEST_1_RIFT_S_PROFILE,
    META_TOUCH_CONTROLLER_QUEST_2_PROFILE,
    SAMSUNG_ODYSSEY_CONTROLLER_PROFILE,
    VALVE_INDEX_CONTROLLER_PROFILE,
    EXT_HAND_INTERACTION_PROFILE,
    FB_HAND_TRACKING_AIM_PROFILE,
];

pub fn get_profiles_from_path(path: String) -> &'static [&'static str] {
    INTERACTION_PROFILES
        .iter()
        .find(|profile| profile.path == path)
        .map_or(&[], |profile| profile.profiles)
}

pub fn get_supported_interaction_profiles(
    supported_extensions: &ExtensionSet,
    enabled_extensions: &mut ExtensionSet,
) -> Vec<&'static str> {
    let mut extensions = Vec::new();
    if supported_extensions.bd_controller_interaction {
        extensions.push(ext_string!(BD_CONTROLLER_INTERACTION_EXTENSION_NAME));
        enabled_extensions.bd_controller_interaction = true;
    }
    if supported_extensions.ext_hp_mixed_reality_controller {
        extensions.push(ext_string!(EXT_HP_MIXED_REALITY_CONTROLLER_EXTENSION_NAME));
        enabled_extensions.ext_hp_mixed_reality_controller = true;
    }
    if supported_extensions.ext_samsung_odyssey_controller {
        extensions.push(ext_string!(EXT_SAMSUNG_ODYSSEY_CONTROLLER_EXTENSION_NAME));
        enabled_extensions.ext_samsung_odyssey_controller = true;
    }
    if supported_extensions.ml_ml2_controller_interaction {
        extensions.push(ext_string!(ML_ML2_CONTROLLER_INTERACTION_EXTENSION_NAME));
        enabled_extensions.ml_ml2_controller_interaction = true;
    }
    if supported_extensions.htc_vive_cosmos_controller_interaction {
        extensions.push(ext_string!(
            HTC_VIVE_COSMOS_CONTROLLER_INTERACTION_EXTENSION_NAME
        ));
        enabled_extensions.htc_vive_cosmos_controller_interaction = true;
    }
    if supported_extensions.htc_vive_focus3_controller_interaction {
        extensions.push(ext_string!(
            HTC_VIVE_FOCUS3_CONTROLLER_INTERACTION_EXTENSION_NAME
        ));
        enabled_extensions.htc_vive_focus3_controller_interaction = true;
    }
    if supported_extensions.fb_touch_controller_pro {
        extensions.push(ext_string!(FB_TOUCH_CONTROLLER_PRO_EXTENSION_NAME));
        enabled_extensions.fb_touch_controller_pro = true;
    }
    if supported_extensions.meta_touch_controller_plus {
        extensions.push(ext_string!(META_TOUCH_CONTROLLER_PLUS_EXTENSION_NAME));
        enabled_extensions.meta_touch_controller_plus = true;
    }
    if supported_extensions.ext_hand_interaction {
        extensions.push(ext_string!(EXT_HAND_INTERACTION_EXTENSION_NAME));
        enabled_extensions.ext_hand_interaction = true;
    }
    if supported_extensions.fb_hand_tracking_aim {
        extensions.push(ext_string!(FB_HAND_TRACKING_AIM_EXTENSION_NAME));
        enabled_extensions.fb_hand_tracking_aim = true;
    }
    extensions
}
