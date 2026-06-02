def element_dimensions(session, element):
    return tuple(session.execute_script("""
        const {devicePixelRatio} = window;
        let {width, height} = arguments[0].getBoundingClientRect();

        return [
          Math.floor(width * devicePixelRatio),
          Math.floor(height * devicePixelRatio),
        ];
        """, args=(element,)))


def take_element_screenshot(session, element_id):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/screenshot".format(
            session_id=session.session_id,
            element_id=element_id,
        )
    )
