use std::rc::Rc;

use webdriver_traits::bidi::{
    EmulationCommand, EmulationResult, ErrorCode,
    emulation::{
        SetForcedColorsModeThemeOverrideParameters, SetForcedColorsModeThemeOverrideResult,
        SetGeolocationOverrideParameters, SetGeolocationOverrideResult,
        SetLocaleOverrideParameters, SetLocaleOverrideResult, SetNetworkConditionsParameters,
        SetNetworkConditionsResult, SetScreenOrientationOverrideParameters,
        SetScreenOrientationOverrideResult, SetScreenSettingsOverrideParameters,
        SetScreenSettingsOverrideResult, SetScriptingEnabledParameters, SetScriptingEnabledResult,
        SetScrollbarTypeOverrideParameters, SetScrollbarTypeOverrideResult,
        SetTimezoneOverrideParameters, SetTimezoneOverrideResult, SetTouchOverrideParameters,
        SetTouchOverrideResult, SetUserAgentOverrideParameters, SetUserAgentOverrideResult,
    },
};

use crate::bidi::{error::BidiResult, remote_end::RemoteEnd};

impl RemoteEnd {
    pub(crate) async fn handle_emulation_command(
        self: Rc<Self>,
        command: EmulationCommand,
    ) -> BidiResult<EmulationResult> {
        match command {
            EmulationCommand::SetForcedColorsModeThemeOverride(cmd) => self
                .handle_emulation_set_forced_colors_mode_theme_overrde(cmd.params)
                .await
                .map(EmulationResult::SetForcedColorsModeThemeOverrideResult),
            EmulationCommand::SetGeolocationOverride(cmd) => self
                .handle_emulation_set_geolocation_override(cmd.params)
                .await
                .map(EmulationResult::SetGeolocationOverrideResult),
            EmulationCommand::SetLocaleOverride(cmd) => self
                .handle_emulation_set_locale_override(cmd.params)
                .await
                .map(EmulationResult::SetLocaleOverrideResult),
            EmulationCommand::SetNetworkConditions(cmd) => self
                .handle_emulation_set_network_conditions(cmd.params)
                .await
                .map(EmulationResult::SetNetworkConditionsResult),
            EmulationCommand::SetScreenOrientationOverride(cmd) => self
                .handle_emulation_set_screen_orientation_override(cmd.params)
                .await
                .map(EmulationResult::SetScreenOrientationOverrideResult),
            EmulationCommand::SetScreenSettingsOverride(cmd) => self
                .handle_emulation_set_screen_settings_override(cmd.params)
                .await
                .map(EmulationResult::SetScreenSettingsOverrideResult),
            EmulationCommand::SetScriptingEnabled(cmd) => self
                .handle_emulation_set_scripting_enabled(cmd.params)
                .await
                .map(EmulationResult::SetScriptingEnabledResult),
            EmulationCommand::SetScrollbarTypeOverride(cmd) => self
                .handle_emulation_set_scrollbar_type_override(cmd.params)
                .await
                .map(EmulationResult::SetScrollbarTypeOverrideResult),
            EmulationCommand::SetTimezoneOverride(cmd) => self
                .handle_emulation_set_timezone_override(cmd.params)
                .await
                .map(EmulationResult::SetTimezoneOverrideResult),
            EmulationCommand::SetTouchOverride(cmd) => self
                .handle_emulation_set_touch_override(cmd.params)
                .await
                .map(EmulationResult::SetTouchOverrideResult),
            EmulationCommand::SetUserAgentOverride(cmd) => self
                .handle_emulation_set_user_agent_override(cmd.params)
                .await
                .map(EmulationResult::SetUserAgentOverrideResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setForcedColorsModeThemeOverride>
    async fn handle_emulation_set_forced_colors_mode_theme_overrde(
        self: Rc<Self>,
        _: SetForcedColorsModeThemeOverrideParameters,
    ) -> BidiResult<SetForcedColorsModeThemeOverrideResult> {
        // TODO: blocked by forced colors mode not implemented
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setGeolocationOverride>
    async fn handle_emulation_set_geolocation_override(
        self: Rc<Self>,
        _: SetGeolocationOverrideParameters,
    ) -> BidiResult<SetGeolocationOverrideResult> {
        // TODO: blocked by geolocation not implemented
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setLocaleOverride>
    async fn handle_emulation_set_locale_override(
        self: Rc<Self>,
        _: SetLocaleOverrideParameters,
    ) -> BidiResult<SetLocaleOverrideResult> {
        // TODO: blocked by `CURRENT_LOCAL` is `OnceLock`
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setNetworkConditions>
    async fn handle_emulation_set_network_conditions(
        self: Rc<Self>,
        _: SetNetworkConditionsParameters,
    ) -> BidiResult<SetNetworkConditionsResult> {
        // TODO: blocked by network condition not implemented
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScreenSettingsOverride>
    async fn handle_emulation_set_screen_settings_override(
        self: Rc<Self>,
        _: SetScreenSettingsOverrideParameters,
    ) -> BidiResult<SetScreenSettingsOverrideResult> {
        // TODO: blocked, need to add a layer
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScreenSettingsOverride>
    async fn handle_emulation_set_scripting_enabled(
        self: Rc<Self>,
        _: SetScriptingEnabledParameters,
    ) -> BidiResult<SetScriptingEnabledResult> {
        // TODO: blocked, need a flag
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScreenOrientationOverride>
    async fn handle_emulation_set_screen_orientation_override(
        self: Rc<Self>,
        _: SetScreenOrientationOverrideParameters,
    ) -> BidiResult<SetScreenOrientationOverrideResult> {
        // TODO: blocked by screen orientation not implemented
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScrollbarTypeOverride>
    async fn handle_emulation_set_scrollbar_type_override(
        self: Rc<Self>,
        _: SetScrollbarTypeOverrideParameters,
    ) -> BidiResult<SetScrollbarTypeOverrideResult> {
        // TODO: blocked by scrollbar type not implemented
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setTimezoneOverride>
    async fn handle_emulation_set_timezone_override(
        self: Rc<Self>,
        _: SetTimezoneOverrideParameters,
    ) -> BidiResult<SetTimezoneOverrideResult> {
        // TODO: blocked, `setRealmTimezoneOverride` is not exported by `mozjs`
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setTouchOverride>
    async fn handle_emulation_set_touch_override(
        self: Rc<Self>,
        _: SetTouchOverrideParameters,
    ) -> BidiResult<SetTouchOverrideResult> {
        // TODO: blocked,by max touch not implemented
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setUserAgentOverride>
    async fn handle_emulation_set_user_agent_override(
        self: Rc<Self>,
        _: SetUserAgentOverrideParameters,
    ) -> BidiResult<SetUserAgentOverrideResult> {
        // TODO: blocked by user agent being global
        // TODO: this may be easy to implement
        Err(ErrorCode::UnknownError.into())
    }
}
