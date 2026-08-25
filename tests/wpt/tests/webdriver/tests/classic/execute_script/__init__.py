import webdriver.protocol as protocol


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session.transport.send(
        "POST",
        "/session/{session_id}/execute/sync".format(**vars(session)),
        body,
        encoder=protocol.Encoder,
        decoder=protocol.Decoder,
        session=session,
    )
