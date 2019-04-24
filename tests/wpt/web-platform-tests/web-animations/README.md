Web Animations Test Suite
=========================

Specification: https://drafts.csswg.org/web-animations/


Guidelines for writing tests
----------------------------

*   Try to follow the spec outline where possible.

    For example, if you want to test setting the start time, you might be
    tempted to put all the tests in:

    > `/web-animations/interfaces/Animation/startTime.html`

    However, in the spec most of the logic is in the &ldquo;Set the animation
    start time&ldquo; procedure in the &ldquo;Timing model&rdquo; section.

    Instead, try something like:

    > *   `/web-animations/timing-model/animations/setting-the-start-time-of-an-animation.html`<br>
    >     Tests all the branches and inputs to the procedure as defined in the
    >     spec (using the `Animation.startTime` API).
    > *   `/web-animations/interfaces/Animation/startTime.html`<br>
    >     Tests API-layer specific issues like mapping unresolved values to
    >      null, etc.

    On that note, two levels of subdirectories is enough even if the spec has
    deeper nesting.

    Note that most of the existing tests in the suite _don't_ do this well yet.
    That's the direction we're heading, however.

*   Test the spec.

    *   If the spec defines a timing calculation that is directly
        reflected in the iteration progress
        (i.e. `anim.effect.getComputedTiming().progress`), test that instead
        of calling `getComputedStyle(elem).marginLeft`.

    *   Likewise, don't add needless tests for `anim.playbackState`.
        The playback state is a calculated value based on other values.
        It's rarely necessary to test directly unless you need, for example,
        to check that a pending task is scheduled (which isn't observable
        elsewhere other than waiting for the corresponding promise to
        complete).

*   Try to keep tests as simple and focused as possible.

    e.g.

      ```javascript
      test(t => {
        const animation = createDiv(t).animate(null);
        assert_class_string(animation, 'Animation', 'Returned object is an Animation');
      }, 'Element.animate() creates an Animation object');
      ```

      ```javascript
      test(t => {
        assert_throws({ name: 'TypeError' }, () => {
          createDiv(t).animate(null, -1);
        });
      }, 'Setting a negative duration throws a TypeError');
      ```

      ```javascript
      promise_test(t => {
        const animation = createDiv(t).animate(null, 100 * MS_PER_SEC);
        return animation.ready.then(() => {
          assert_greater_than(animation.startTime, 0, 'startTime when running');
        });
      }, 'startTime is resolved when running');
      ```

    If you're generating complex test loops and factoring out utility functions
    that affect the logic of the test (other than, say, simple assertion utility
    functions), you're probably doing it wrong.

    It should be possible to understand exactly what the test is doing at a
    glance without having to scroll up and down the test file and refer to
    other files.

    See Justin Searls' presentation, [&ldquo;How to stop hating your
    tests&rdquo;](http://blog.testdouble.com/posts/2015-11-16-how-to-stop-hating-your-tests.html)
    for some tips on making your tests simpler.

*   Assume tests will run on under-performing hardware where the time between
    animation frames might run into 10s of seconds.
    As a result, animations that are expected to still be running during
    the test should be at least 100s in length.

*   Avoid using `GLOBAL_CONSTS` that make the test harder to read.
    It's fine to repeat the the same parameter values like `100 * MS_PER_SEC`
    over and over again since it makes it easy to read and debug a test in
    isolation.
    Remember, even if we do need to make all tests take, say 200s each, text
    editors are very good at search and replace.

*   Use the `assert_times_equal` assertion for comparing times returned from
    the API. This asserts that the time values are equal using a tolerance
    based on the precision recommended in the spec. This tolerance is applied
    to *both* of the values being compared. That is, it effectively allows
    double the epsilon that is used when comparing with an absolute value.

    For comparing a time value returned from the API to an absolute value, use
    `assert_time_equals_literal`. This tests that the actual value is equal to
    the expected value within the precision recommended in the spec.

    Both `assert_times_equal` and `assert_time_equals_literal` are defined in a
    way that implementations can override them to meet their own precision
    requirements.

*   There are quite a few bad tests in the repository. We're learning as
    we go. Don't just copy them blindly&mdash;please fix them!
