def viewport_dimensions(session):
    return tuple(session.execute_script("""
        let {devicePixelRatio, innerHeight, innerWidth} = window;

        return [
          Math.floor(innerWidth * devicePixelRatio),
          Math.floor(innerHeight * devicePixelRatio)
        ];
        """))
