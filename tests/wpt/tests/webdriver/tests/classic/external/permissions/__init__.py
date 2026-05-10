def query_permissions(session, name):
    """Query the status of a permission.

    Uses the 'navigator.permissions.query()' API.

    :param session: WebDriver session object.
    :param name: Permission name to query (e.g., "geolocation", "camera").

    :returns: Result object with 'status' ('success' or 'error') and 'value'
              (permission state or error message).
    """
    return session.execute_async_script(
        """
        var [name, resolve] = arguments;

        navigator.permissions.query({ name })
          .then(function(value) {
              resolve({ status: 'success', value: value && value.state });
            }, function(error) {
              resolve({ status: 'error', value: error && error.message });
            });
    """,
        args=[name],
    )


def set_permissions(session, parameters):
    """Set permission states.

    Uses the WebDriver '/session/{session_id}/permissions' endpoint.

    :param session: WebDriver session object.
    :param parameters: Permission parameters to set, typically containing permission
                       descriptors with their desired states.

    :returns: Response from the WebDriver endpoint.
    """
    return session.transport.send(
        "POST", "/session/{session_id}/permissions".format(**vars(session)), parameters
    )
