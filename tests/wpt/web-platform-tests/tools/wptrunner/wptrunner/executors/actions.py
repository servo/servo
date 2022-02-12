
class ClickAction(object):
    name = "click"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        selector = payload["selector"]
        element = self.protocol.select.element_by_selector(selector)
        self.logger.debug("Clicking element: %s" % selector)
        self.protocol.click.element(element)


class DeleteAllCookiesAction(object):
    name = "delete_all_cookies"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Deleting all cookies")
        self.protocol.cookies.delete_all_cookies()


class SendKeysAction(object):
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


class MinimizeWindowAction(object):
    name = "minimize_window"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        return self.protocol.window.minimize()


class SetWindowRectAction(object):
    name = "set_window_rect"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        rect = payload["rect"]
        self.protocol.window.set_rect(rect)


class ActionSequenceAction(object):
    name = "action_sequence"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        # TODO: some sort of shallow error checking
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


class GenerateTestReportAction(object):
    name = "generate_test_report"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        message = payload["message"]
        self.logger.debug("Generating test report: %s" % message)
        self.protocol.generate_test_report.generate_test_report(message)

class SetPermissionAction(object):
    name = "set_permission"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        permission_params = payload["permission_params"]
        descriptor = permission_params["descriptor"]
        name = descriptor["name"]
        state = permission_params["state"]
        one_realm = permission_params.get("oneRealm", False)
        self.logger.debug("Setting permission %s to %s, oneRealm=%s" % (name, state, one_realm))
        self.protocol.set_permission.set_permission(descriptor, state, one_realm)

class AddVirtualAuthenticatorAction(object):
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

class RemoveVirtualAuthenticatorAction(object):
    name = "remove_virtual_authenticator"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        self.logger.debug("Removing virtual authenticator %s" % authenticator_id)
        return self.protocol.virtual_authenticator.remove_virtual_authenticator(authenticator_id)


class AddCredentialAction(object):
    name = "add_credential"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        credential = payload["credential"]
        self.logger.debug("Adding credential to virtual authenticator %s " % authenticator_id)
        return self.protocol.virtual_authenticator.add_credential(authenticator_id, credential)

class GetCredentialsAction(object):
    name = "get_credentials"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        self.logger.debug("Getting credentials from virtual authenticator %s " % authenticator_id)
        return self.protocol.virtual_authenticator.get_credentials(authenticator_id)

class RemoveCredentialAction(object):
    name = "remove_credential"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        credential_id = payload["credential_id"]
        self.logger.debug("Removing credential %s from authenticator %s" % (credential_id, authenticator_id))
        return self.protocol.virtual_authenticator.remove_credential(authenticator_id, credential_id)

class RemoveAllCredentialsAction(object):
    name = "remove_all_credentials"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        self.logger.debug("Removing all credentials from authenticator %s" % authenticator_id)
        return self.protocol.virtual_authenticator.remove_all_credentials(authenticator_id)

class SetUserVerifiedAction(object):
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

class SetSPCTransactionModeAction(object):
    name = "set_spc_transaction_mode"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        mode = payload["mode"]
        self.logger.debug("Setting SPC transaction mode to %s" % mode)
        return self.protocol.spc_transactions.set_spc_transaction_mode(mode)

actions = [ClickAction,
           DeleteAllCookiesAction,
           SendKeysAction,
           MinimizeWindowAction,
           SetWindowRectAction,
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
           SetSPCTransactionModeAction]
