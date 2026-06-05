use rustenium_bidi_definitions::emulation::commands::EmulationCommand;

use crate::{error::WebDriverBidiError, handler::Handler, model::EmulationResult};

impl Handler {
    pub(super) async fn handle_emulation(
        &self,
        cmd: EmulationCommand,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        match cmd {
            EmulationCommand::SetForcedColorsModeThemeOverride(
                set_forced_colors_mode_theme_override,
            ) => {
                self.handle_emulation_set_forced_colors_mode_theme_override()
                    .await
            },
            EmulationCommand::SetGeolocationOverride(set_geolocation_override) => {
                self.handle_emulation_set_geolocation_override().await
            },
            EmulationCommand::SetLocaleOverride(set_locale_override) => {
                self.handle_emulation_set_locale_override().await
            },
            EmulationCommand::SetNetworkConditions(set_network_conditions) => {
                self.handle_emulation_set_network_conditions().await
            },
            EmulationCommand::SetScreenOrientationOverride(set_screen_orientation_override) => {
                self.handle_emulation_set_screen_orientation_override()
                    .await
            },
            EmulationCommand::SetUserAgentOverride(set_user_agent_override) => {
                self.handle_emulation_set_user_agent_override().await
            },
            EmulationCommand::SetScriptingEnabled(set_scripting_enabled) => {
                self.handle_emulation_set_scripting_enabled().await
            },
            EmulationCommand::SetTimezoneOverride(set_timezone_override) => {
                self.handle_emulation_set_timezone_override().await
            },
        }
    }

    async fn handle_emulation_set_forced_colors_mode_theme_override(
        &self,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        todo!()
    }

    async fn handle_emulation_set_geolocation_override(
        &self,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        todo!()
    }

    async fn handle_emulation_set_locale_override(
        &self,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        todo!()
    }

    async fn handle_emulation_set_network_conditions(
        &self,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        todo!()
    }

    async fn handle_emulation_set_screen_orientation_override(
        &self,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        todo!()
    }

    async fn handle_emulation_set_user_agent_override(
        &self,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        todo!()
    }

    async fn handle_emulation_set_scripting_enabled(
        &self,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        todo!()
    }

    async fn handle_emulation_set_timezone_override(
        &self,
    ) -> Result<EmulationResult, WebDriverBidiError> {
        todo!()
    }
}
