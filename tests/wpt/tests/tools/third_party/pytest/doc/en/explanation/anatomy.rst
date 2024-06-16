.. _test-anatomy:

Anatomy of a test
=================

In the simplest terms, a test is meant to look at the result of a particular
behavior, and make sure that result aligns with what you would expect.
Behavior is not something that can be empirically measured, which is why writing
tests can be challenging.

"Behavior" is the way in which some system **acts in response** to a particular
situation and/or stimuli. But exactly *how* or *why* something is done is not
quite as important as *what* was done.

You can think of a test as being broken down into four steps:

1. **Arrange**
2. **Act**
3. **Assert**
4. **Cleanup**

**Arrange** is where we prepare everything for our test. This means pretty
much everything except for the "**act**". It's lining up the dominoes so that
the **act** can do its thing in one, state-changing step. This can mean
preparing objects, starting/killing services, entering records into a database,
or even things like defining a URL to query, generating some credentials for a
user that doesn't exist yet, or just waiting for some process to finish.

**Act** is the singular, state-changing action that kicks off the **behavior**
we want to test. This behavior is what carries out the changing of the state of
the system under test (SUT), and it's the resulting changed state that we can
look at to make a judgement about the behavior. This typically takes the form of
a function/method call.

**Assert** is where we look at that resulting state and check if it looks how
we'd expect after the dust has settled. It's where we gather evidence to say the
behavior does or does not align with what we expect. The ``assert`` in our test
is where we take that measurement/observation and apply our judgement to it. If
something should be green, we'd say ``assert thing == "green"``.

**Cleanup** is where the test picks up after itself, so other tests aren't being
accidentally influenced by it.

At its core, the test is ultimately the **act** and **assert** steps, with the
**arrange** step only providing the context. **Behavior** exists between **act**
and **assert**.
