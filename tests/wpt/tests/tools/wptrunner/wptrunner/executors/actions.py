# mypy: allow-untyped-defs

class ClickAction:
    name = "click"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        selector = payload["selector"]
        element = self.protocol.select.element_by_selector(selector)
        self.logger.debug("Clicking element: %s" % selector)
        self.protocol.click.element(element)


class DeleteAllCookiesAction:
    name = "delete_all_cookies"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Deleting all cookies")
        self.protocol.cookies.delete_all_cookies()


class GetAllCookiesAction:
    name = "get_all_cookies"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Getting all cookies")
        return self.protocol.cookies.get_all_cookies()


class GetComputedLabelAction:
    name = "get_computed_label"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        selector = payload["selector"]
        element = self.protocol.select.element_by_selector(selector)
        self.logger.debug("Getting computed label for element: %s" % element)
        return self.protocol.accessibility.get_computed_label(element)


class GetComputedRoleAction:
    name = "get_computed_role"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        selector = payload["selector"]
        element = self.protocol.select.element_by_selector(selector)
        self.logger.debug("Getting computed role for element: %s" % element)
        return self.protocol.accessibility.get_computed_role(element)


class GetNamedCookieAction:
    name = "get_named_cookie"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        name = payload["name"]
        self.logger.debug("Getting cookie named %s" % name)
        return self.protocol.cookies.get_named_cookie(name)


class SendKeysAction:
    name = "send_keys"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        selector = payload["selector"]
        keys = payload["keys"]
        element = self.protocol.select.element_by_selector(selector)
        self.logger.debug("Sending keys to element: %s" % selector)
        self.protocol.send_keys.send_keys(element, keys)


class MinimizeWindowAction:
    name = "minimize_window"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        return self.protocol.window.minimize()


class SetWindowRectAction:
    name = "set_window_rect"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        rect = payload["rect"]
        self.protocol.window.set_rect(rect)

class GetWindowRectAction:
    name = "get_window_rect"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        return self.protocol.window.get_rect()

class ActionSequenceAction:
    name = "action_sequence"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol
        self.requires_state_reset = False

    def __call__(self, payload):
        # TODO: some sort of shallow error checking
        if self.requires_state_reset:
            self.reset()
        self.requires_state_reset = True
        actions = payload["actions"]
        for actionSequence in actions:
            if actionSequence["type"] == "pointer":
                for action in actionSequence["actions"]:
                    if (action["type"] == "pointerMove" and
                        isinstance(action["origin"], dict)):
                        action["origin"] = self.get_element(action["origin"]["selector"])
        self.protocol.action_sequence.send_actions({"actions": actions})

    def get_element(self, element_selector):
        return self.protocol.select.element_by_selector(element_selector)

    def reset(self):
        self.protocol.action_sequence.release()
        self.requires_state_reset = False


class GenerateTestReportAction:
    name = "generate_test_report"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        message = payload["message"]
        self.logger.debug("Generating test report: %s" % message)
        self.protocol.generate_test_report.generate_test_report(message)

class SetPermissionAction:
    name = "set_permission"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        permission_params = payload["permission_params"]
        descriptor = permission_params["descriptor"]
        name = descriptor["name"]
        state = permission_params["state"]
        self.logger.debug("Setting permission %s to %s" % (name, state))
        self.protocol.set_permission.set_permission(descriptor, state)

class AddVirtualAuthenticatorAction:
    name = "add_virtual_authenticator"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Adding virtual authenticator")
        config = payload["config"]
        authenticator_id = self.protocol.virtual_authenticator.add_virtual_authenticator(config)
        self.logger.debug("Authenticator created with ID %s" % authenticator_id)
        return authenticator_id

class RemoveVirtualAuthenticatorAction:
    name = "remove_virtual_authenticator"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        self.logger.debug("Removing virtual authenticator %s" % authenticator_id)
        return self.protocol.virtual_authenticator.remove_virtual_authenticator(authenticator_id)


class AddCredentialAction:
    name = "add_credential"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        credential = payload["credential"]
        self.logger.debug("Adding credential to virtual authenticator %s " % authenticator_id)
        return self.protocol.virtual_authenticator.add_credential(authenticator_id, credential)

class GetCredentialsAction:
    name = "get_credentials"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        self.logger.debug("Getting credentials from virtual authenticator %s " % authenticator_id)
        return self.protocol.virtual_authenticator.get_credentials(authenticator_id)

class RemoveCredentialAction:
    name = "remove_credential"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        credential_id = payload["credential_id"]
        self.logger.debug("Removing credential %s from authenticator %s" % (credential_id, authenticator_id))
        return self.protocol.virtual_authenticator.remove_credential(authenticator_id, credential_id)

class RemoveAllCredentialsAction:
    name = "remove_all_credentials"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        self.logger.debug("Removing all credentials from authenticator %s" % authenticator_id)
        return self.protocol.virtual_authenticator.remove_all_credentials(authenticator_id)

class SetUserVerifiedAction:
    name = "set_user_verified"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        uv = payload["uv"]
        self.logger.debug(
            "Setting user verified flag on authenticator %s to %s" % (authenticator_id, uv["isUserVerified"]))
        return self.protocol.virtual_authenticator.set_user_verified(authenticator_id, uv)

class SetSPCTransactionModeAction:
    name = "set_spc_transaction_mode"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        mode = payload["mode"]
        self.logger.debug("Setting SPC transaction mode to %s" % mode)
        return self.protocol.spc_transactions.set_spc_transaction_mode(mode)

class SetRPHRegistrationModeAction:
    name = "set_rph_registration_mode"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        mode = payload["mode"]
        self.logger.debug("Setting RPH registration mode to %s" % mode)
        return self.protocol.rph_registrations.set_rph_registration_mode(mode)

class CancelFedCMDialogAction:
    name = "cancel_fedcm_dialog"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Canceling FedCM dialog")
        return self.protocol.fedcm.cancel_fedcm_dialog()

class ClickFedCMDialogButtonAction:
    name = "click_fedcm_dialog_button"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        dialog_button = payload["dialog_button"]
        self.logger.debug(f"Clicking FedCM dialog button: {dialog_button}")
        return self.protocol.fedcm.click_fedcm_dialog_button()

class SelectFedCMAccountAction:
    name = "select_fedcm_account"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        account_index = payload["account_index"]
        self.logger.debug(f"Selecting FedCM account of index: {account_index}")
        return self.protocol.fedcm.select_fedcm_account(account_index)

class GetFedCMAccountListAction:
    name = "get_fedcm_account_list"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Getting FedCM account list")
        return self.protocol.fedcm.get_fedcm_account_list()

class GetFedCMDialogTitleAction:
    name = "get_fedcm_dialog_title"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Getting FedCM dialog title")
        return self.protocol.fedcm.get_fedcm_dialog_title()

class GetFedCMDialogTypeAction:
    name = "get_fedcm_dialog_type"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Getting FedCM dialog type")
        return self.protocol.fedcm.get_fedcm_dialog_type()

class SetFedCMDelayEnabledAction:
    name = "set_fedcm_delay_enabled"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        enabled = payload["enabled"]
        self.logger.debug("Setting FedCM delay enabled as %s" % enabled)
        return self.protocol.fedcm.set_fedcm_delay_enabled(enabled)

class ResetFedCMCooldownAction:
    name = "reset_fedcm_cooldown"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Resetting FedCM cooldown")
        return self.protocol.fedcm.reset_fedcm_cooldown()


class CreateVirtualSensorAction:
    name = "create_virtual_sensor"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        sensor_type = payload["sensor_type"]
        sensor_params = payload["sensor_params"]
        self.logger.debug("Creating %s sensor with %s values" % (sensor_type, sensor_params))
        return self.protocol.virtual_sensor.create_virtual_sensor(sensor_type, sensor_params)


class UpdateVirtualSensorAction:
    name = "update_virtual_sensor"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        sensor_type = payload["sensor_type"]
        reading = payload["reading"]
        self.logger.debug("Updating %s sensor with new readings: %s" % (sensor_type, reading))
        return self.protocol.virtual_sensor.update_virtual_sensor(sensor_type, reading)


class RemoveVirtualSensorAction:
    name = "remove_virtual_sensor"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        sensor_type = payload["sensor_type"]
        self.logger.debug("Removing %s sensor" % sensor_type)
        return self.protocol.virtual_sensor.remove_virtual_sensor(sensor_type)


class GetVirtualSensorInformationAction:
    name = "get_virtual_sensor_information"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        sensor_type = payload["sensor_type"]
        self.logger.debug("Requesting information from %s sensor" % sensor_type)
        return self.protocol.virtual_sensor.get_virtual_sensor_information(sensor_type)


actions = [ClickAction,
           DeleteAllCookiesAction,
           GetAllCookiesAction,
           GetNamedCookieAction,
           GetComputedLabelAction,
           GetComputedRoleAction,
           SendKeysAction,
           MinimizeWindowAction,
           SetWindowRectAction,
           GetWindowRectAction,
           ActionSequenceAction,
           GenerateTestReportAction,
           SetPermissionAction,
           AddVirtualAuthenticatorAction,
           RemoveVirtualAuthenticatorAction,
           AddCredentialAction,
           GetCredentialsAction,
           RemoveCredentialAction,
           RemoveAllCredentialsAction,
           SetUserVerifiedAction,
           SetSPCTransactionModeAction,
           SetRPHRegistrationModeAction,
           CancelFedCMDialogAction,
           ClickFedCMDialogButtonAction,
           SelectFedCMAccountAction,
           GetFedCMAccountListAction,
           GetFedCMDialogTitleAction,
           GetFedCMDialogTypeAction,
           SetFedCMDelayEnabledAction,
           ResetFedCMCooldownAction,
           CreateVirtualSensorAction,
           UpdateVirtualSensorAction,
           RemoveVirtualSensorAction,
           GetVirtualSensorInformationAction]
