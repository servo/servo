def element_rect(session, element):
    return session.execute_script("""
        let {devicePixelRatio} = window;
        let {left, top, width, height} = arguments[0].getBoundingClientRect();

        return {
            x: Math.floor((left + window.pageXOffset) * devicePixelRatio),
            y: Math.floor((top + window.pageYOffset) * devicePixelRatio),
            width: Math.floor(width * devicePixelRatio),
            height: Math.floor(height * devicePixelRatio),
        };
        """, args=(element,))
