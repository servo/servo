def document_dimensions(session):
    return tuple(session.execute_script("""
        let devicePixelRatio = window.devicePixelRatio;
        let rect = document.documentElement.getBoundingClientRect();
        return [Math.floor(rect.width * devicePixelRatio), Math.floor(rect.height * devicePixelRatio)];
        """))
