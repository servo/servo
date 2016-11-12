from servo_tidy.tidy import LintRunner

class Lint(LintRunner):
    def run(self):
        for _ in [None]:
            yield ('path', 0, 'foobar')
