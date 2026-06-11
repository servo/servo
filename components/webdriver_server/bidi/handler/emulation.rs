use servo_webdriver::bidi::{
    EmulationCommand, EmulationResult,
    emulation::{
        SetForcedColorsModeThemeOverrideParameters, SetForcedColorsModeThemeOverrideResult,
        SetGeolocationOverrideParameters, SetGeolocationOverrideResult,
        SetLocaleOverrideParameters, SetLocaleOverrideResult, SetNetworkConditionsParameters,
        SetNetworkConditionsResult, SetScreenOrientationOverrideParameters,
        SetScreenOrientationOverrideResult, SetScriptingEnabledParameters,
        SetScriptingEnabledResult, SetTimezoneOverrideParameters, SetTimezoneOverrideResult,
        SetUserAgentOverrideParameters, SetUserAgentOverrideResult,
    },
};

use crate::bidi::{error::WebDriverBidiError, handler::Handler};

impl Handler {
    pub(super) async fn handle_emulation(
        &self,
        cmd: EmulationCommand,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        match cmd {
            EmulationCommand::SetForcedColorsModeThemeOverride(cmd) => self
                .handle_emulation_set_forced_colors_mode_theme_override(cmd.params)
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
            EmulationCommand::SetUserAgentOverride(cmd) => self
                .handle_emulation_set_user_agent_override(cmd.params)
                .await
                .map(EmulationResult::SetUserAgentOverrideResult),
            EmulationCommand::SetScriptingEnabled(cmd) => self
                .handle_emulation_set_scripting_enabled(cmd.params)
                .await
                .map(EmulationResult::SetScriptingEnabledResult),
            EmulationCommand::SetTimezoneOverride(cmd) => self
                .handle_emulation_set_timezone_override(cmd.params)
                .await
                .map(EmulationResult::SetTimezoneOverrideResult),
            EmulationCommand::SetScreenSettingsOverride(set_screen_settings_override) => todo!(),
            EmulationCommand::SetScrollbarTypeOverride(set_scrollbar_type_override) => todo!(),
            EmulationCommand::SetTouchOverride(set_touch_override) => todo!(),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setForcedColorsModeThemeOverride>
    async fn handle_emulation_set_forced_colors_mode_theme_override(
        &self,
        command_parameters: SetForcedColorsModeThemeOverrideParameters,
    ) -> Result<SetForcedColorsModeThemeOverrideResult, WebDriverBidiError> {
        // 1. Let theme be command parameters["theme"].

        // 2. If theme is null, set theme to unset.

        // 3. If the implementation does not support setting theme, then return error with error code unsupported operation.

        // 4. Let affected navigables be the result of trying to store WebDriver configuration forced colors mode theme override configuration theme for command parameters.

        // 5. For each navigable of affected navigables:

        // 6. Update emulated forced colors theme for navigable.

        // 7. Return success with data null.
        Ok(SetForcedColorsModeThemeOverrideResult {
            extensible: Default::default(),
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setGeolocationOverride>
    async fn handle_emulation_set_geolocation_override(
        &self,
        command_parameters: SetGeolocationOverrideParameters,
    ) -> Result<SetGeolocationOverrideResult, WebDriverBidiError> {
        // 1. If command parameters contains "coordinates" and command parameters["coordinates"] contains "altitudeAccuracy" and command parameters["coordinates"] doesn’t contain "altitude", return error with error code invalid argument.

        // 2. If command parameters contains "error":

        // 2.1. Assert command parameters["error"]["type"] equals "positionUnavailable".

        // 2.2. Let emulated position data be a map matching GeolocationPositionError production, with code field set to POSITION_UNAVAILABLE and message field set to the empty string.

        // 3. Otherwise, let emulated position data be command parameters["coordinates"].

        // 4. If emulated position data is null, set emulated position data to unset.

        // 5. Let affected navigables be the result of trying to store WebDriver configuration geolocation override configuration emulated position data for command parameters.

        // 6. For each navigable of affected navigables:

        // 6.1. Update geolocation override for navigable.

        // 7. Return success with data null.
        Ok(SetGeolocationOverrideResult {
            extensible: Default::default(),
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setLocaleOverride>
    async fn handle_emulation_set_locale_override(
        &self,
        command_parameters: SetLocaleOverrideParameters,
    ) -> Result<SetLocaleOverrideResult, WebDriverBidiError> {
        // 1. If command parameters contains "userContexts" and command parameters contains "contexts", return error with error code invalid argument.

        // 2. If command parameters doesn’t contain "userContexts" and command parameters doesn’t contain "contexts", return error with error code invalid argument.

        // 3. Let emulated locale be command parameters["locale"].

        // 4. If emulated locale is not null and IsStructurallyValidLanguageTag(emulated locale) returns false, return error with error code invalid argument.

        // 5. Let navigables be a set.

        // 6. If the contexts field of command parameters is present:

        // 6.1. Let navigables be the result of trying to get valid top-level traversables by ids with command parameters["contexts"].

        // 7. Otherwise:

        // 7.1. Assert the userContexts field of command parameters is present.

        // 7.2. Let user contexts be the result of trying to get valid user contexts with command parameters["userContexts"].

        // 7.3. For each user context of user contexts:

        // 7.3.1. If emulated locale is null, remove user context from locale overrides map.

        // 7.3.2. Otherwise, set locale overrides map[user context] to emulated locale.

        // 7.3.3. For each top-level traversable of the list of all top-level traversables whose associated user context is user context:

        // 7.3.3.1. Append top-level traversable to navigables.

        // 8. For each navigable of navigables:

        // 8.1. If emulated locale is null, remove navigable from locale overrides map.

        // 8.2. Otherwise, set locale overrides map[navigable] to emulated locale.

        // 9. Return [success] with data null.
        Ok(SetLocaleOverrideResult {
            extensible: Default::default(),
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setNetworkConditions>
    async fn handle_emulation_set_network_conditions(
        &self,
        command_parameters: SetNetworkConditionsParameters,
    ) -> Result<SetNetworkConditionsResult, WebDriverBidiError> {
        // If command parameters contains "userContexts" and command parameters contains "context", return error with error code invalid argument.

        // Let emulated network conditions be null.

        // If command parameters["networkConditions"] is not null and command parameters["networkConditions"]["type"] equals "offline", set emulated network conditions to a new emulated network conditions struct with offline set to true.

        // If the contexts field of command parameters is present:

        // Let navigables be the result of trying to get valid top-level traversables by ids with command parameters["contexts"].

        // For each navigable of navigables:

        // If emulated network conditions is null, remove navigable from session’s emulated network conditions’s navigable network conditions

        // Otherwise, set session’s emulated network conditions’s navigable network conditions[navigable] to emulated network conditions.

        // If the userContexts field of command parameters is present:

        // Let user contexts be the result of trying to get valid user contexts with command parameters["userContexts"].

        // For each user context of user contexts:

        // If emulated network conditions is null, remove user context from session’s emulated network conditions’s user context network conditions.

        // Otherwise, set session’s emulated network conditions’s user context network conditions[user context] to emulated network conditions.

        // If command parameters doesn’t contain "userContexts" and command parameters doesn’t contain "context", set session’s emulated network conditions’s default network conditions to emulated network conditions.

        // Apply network conditions.

        // Return success with data null.
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScreenOrientationOverride>
    async fn handle_emulation_set_screen_orientation_override(
        &self,
        command_parameters: SetScreenOrientationOverrideParameters,
    ) -> Result<SetScreenOrientationOverrideResult, WebDriverBidiError> {
        // If the implementation is unable to adjust the screen orientations parameters with the given command parameters for any reason, return error with error code unsupported operation.

        // If command parameters contains "userContexts" and command parameters contains "contexts", return error with error code invalid argument.

        // If command parameters doesn’t contain "userContexts" and command parameters doesn’t contain "contexts", return error with error code invalid argument.

        // Let emulated screen orientation be command parameters["screenOrientation"].

        // Let navigables be a set.

        // If the contexts field of command parameters is present:

        // Let navigables be the result of trying to get valid top-level traversables by ids with command parameters["contexts"].

        // Otherwise, if the userContexts field of command parameters is present:

        // Let user contexts be the result of trying to get valid user contexts with command parameters["userContexts"].

        // For each user context of user contexts:

        // If emulated screen orientation is null, remove user context from screen orientation overrides map.

        // Otherwise, set screen orientation overrides map[user context] to emulated screen orientation.

        // For each top-level traversable of the list of all top-level traversables whose associated user context is user context:

        // Append top-level traversable to navigables.

        // For each navigable of navigables:

        // Let user context be navigable’s associated user context.

        // If emulated screen orientation is null and screen orientation overrides map contains user context, set emulated screen orientation with navigable and screen orientation overrides map[user context].

        // Otherwise, set emulated screen orientation with navigable and emulated screen orientation.

        // Return success with data null.
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setUserAgentOverride>
    async fn handle_emulation_set_user_agent_override(
        &self,
        command_parameters: SetUserAgentOverrideParameters,
    ) -> Result<SetUserAgentOverrideResult, WebDriverBidiError> {
        //
        // If command parameters contains "userContexts" and command parameters contains "contexts", return error with error code invalid argument.

        // Let emulated user agent be command parameters["userAgent"].

        // If command parameters contains "contexts":

        // Let navigables be the result of trying to get valid top-level traversables by ids with command parameters["contexts"].

        // For each navigable of navigables:

        // If emulated user agent is null, remove navigable from session’s emulated user agent’s navigable user agent.

        // Otherwise, set session’s emulated user agent’s navigable user agent[navigable] to emulated user agent.

        // Return success with data null.

        // If command parameters contains "userContexts":

        // Let user contexts be the result of trying to get valid user contexts with command parameters["userContexts"].

        // For each user context of user contexts:

        // If emulated user agent is null, remove user context from session’s emulated user agent’s user context user agent.

        // Otherwise, set session’s emulated user agent’s user context user agent[user context] to emulated user agent.

        // Return success with data null.

        // Set session’s emulated user agent’s default user agent to emulated user agent.

        // Return success with data null.

        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScriptingEnabled>
    async fn handle_emulation_set_scripting_enabled(
        &self,
        command_parameters: SetScriptingEnabledParameters,
    ) -> Result<SetScriptingEnabledResult, WebDriverBidiError> {
        // If command parameters contains "userContexts" and command parameters contains "contexts", return error with error code invalid argument.

        // If command parameters doesn’t contain "userContexts" and command parameters doesn’t contain "contexts", return error with error code invalid argument.

        // Let emulated scripting enabled status be command parameters["enabled"].

        // If the contexts field of command parameters is present:

        // Let navigables be the result of trying to get valid top-level traversables by ids with command parameters["contexts"].

        // For each navigable of navigables:

        // If emulated scripting enabled status is null, remove navigable from scripting enabled overrides map.

        // Otherwise, set scripting enabled overrides map[navigable] to emulated scripting enabled status.

        // If the userContexts field of command parameters is present:

        // Let user contexts be the result of trying to get valid user contexts with command parameters["userContexts"].

        // For each user context of user contexts:

        // If emulated scripting enabled status is null, remove user context from scripting enabled overrides map.

        // Otherwise set scripting enabled overrides map[user context] to emulated scripting enabled status.

        // Return success with data null.

        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setTimezoneOverride>
    async fn handle_emulation_set_timezone_override(
        &self,
        command_parameters: SetTimezoneOverrideParameters,
    ) -> Result<SetTimezoneOverrideResult, WebDriverBidiError> {
        // If command parameters contains "userContexts" and command parameters contains "contexts", return error with error code invalid argument.

        // If command parameters doesn’t contain "userContexts" and command parameters doesn’t contain "contexts", return error with error code invalid argument.

        // Let emulated timezone be command parameters["timezone"].

        // If emulated timezone is not null and IsTimeZoneOffsetString(emulated timezone) returns false and AvailableNamedTimeZoneIdentifiers does not contain emulated timezone, return error with error code invalid argument.

        // Let navigables be a set.

        // If the contexts field of command parameters is present:

        // Let navigables be the result of trying to get valid top-level traversables by ids with command parameters["contexts"].

        // Otherwise:

        // Assert the userContexts field of command parameters is present.

        // Let user contexts be the result of trying to get valid user contexts with command parameters["userContexts"].

        // For each user context of user contexts:

        // If emulated timezone is null, remove user context from timezone overrides map.

        // Otherwise, set timezone overrides map[user context] to emulated timezone.

        // For each top-level traversable of the list of all top-level traversables whose associated user context is user context:

        // Append top-level traversable to navigables.

        // For each navigable of navigables:

        // If emulated timezone is null, remove navigable from timezone overrides map.

        // Otherwise, set timezone overrides map[navigable] to emulated timezone.

        // 9. Return [success] with data null.
        Ok(SetTimezoneOverrideResult {
            extensible: Default::default(),
        })
    }
}
