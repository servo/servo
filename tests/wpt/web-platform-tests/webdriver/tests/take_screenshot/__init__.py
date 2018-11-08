def document_dimensions(session):
    return tuple(session.execute_script("""
        let {devicePixelRatio} = window;
        let {width, height} = document.documentElement.getBoundingClientRect();
        return [Math.floor(width * devicePixelRatio), Math.floor(height * devicePixelRatio)];
        """))
