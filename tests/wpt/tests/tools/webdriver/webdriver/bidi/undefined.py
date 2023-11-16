class Undefined:
    def __init__(self) -> None:
        raise RuntimeError('Import UNDEFINED instead.')


UNDEFINED = Undefined.__new__(Undefined)
