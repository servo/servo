def element_dimensions(session, element):
    return tuple(session.execute_script("""
        const {devicePixelRatio} = window;
        let {width, height} = arguments[0].getBoundingClientRect();

        return [
          Math.floor(width * devicePixelRatio),
          Math.floor(height * devicePixelRatio),
        ];
        """, args=(element,)))
