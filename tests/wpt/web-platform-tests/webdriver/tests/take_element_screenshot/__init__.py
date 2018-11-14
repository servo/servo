def element_rect(session, element):
    return session.execute_script("""
        let devicePixelRatio = window.devicePixelRatio;
        let rect = arguments[0].getBoundingClientRect();

        return {
            x: Math.floor((rect.left + window.pageXOffset) * devicePixelRatio),
            y: Math.floor((rect.top + window.pageYOffset) * devicePixelRatio),
            width: Math.floor(rect.width * devicePixelRatio),
            height: Math.floor(rect.height * devicePixelRatio),
        };
        """, args=(element,))
