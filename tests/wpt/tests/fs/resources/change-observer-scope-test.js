// This script depends on the following scripts:
//    resources/test-helpers.js

// A helper class for WPTs testing FileSystemObserver scope behavior.
//
// Sets up a `watched_handle` for the test to watch. Provides the
// `in_scope_paths()` and `out_of_scope_paths()` async iterators to get paths
// that are in scope or out of scope of the `watched_path` respectively.
class ScopeTest {
  #test_dir_handle;

  #watched_handle;
  #out_of_scope_directory;

  #child_dir_name;
  #child_dir_handle;

  #setup_promise_and_resolvers = Promise.withResolvers();

  constructor(test, test_dir_handle) {
    test.add_cleanup(async () => {
      await this.#setup_promise_and_resolvers.promise;
      this.#watched_handle.remove({recursive: true});
      this.#out_of_scope_directory.remove({recursive: true});
    });

    this.#test_dir_handle = test_dir_handle;

    this.#setup();
  }

  async watched_handle() {
    await this.#setup_promise_and_resolvers.promise;
    return this.#watched_handle;
  }

  async * in_scope_paths(recursive) {
    await this.#setup_promise_and_resolvers.promise;

    yield new ScopeTestPath(this.#watched_handle, [])

    if (recursive) {
      yield new ScopeTestPath(this.#child_dir_handle, [this.#child_dir_name]);
    }
  }

  async * out_of_scope_paths(recursive) {
    await this.#setup_promise_and_resolvers.promise;

    yield new ScopeTestPath(this.#out_of_scope_directory, [])

    if (!recursive) {
      yield new ScopeTestPath(this.#child_dir_handle, [this.#child_dir_name]);
    }
  }

  async #setup() {
    this.#watched_handle = await this.#test_dir_handle.getDirectoryHandle(
        getUniqueName(), {create: true});

    this.#child_dir_name = getUniqueName();
    this.#child_dir_handle = await this.#watched_handle.getDirectoryHandle(
        this.#child_dir_name, {create: true});

    this.#out_of_scope_directory =
        await this.#test_dir_handle.getDirectoryHandle(
            getUniqueName(), {create: true});

    this.#setup_promise_and_resolvers.resolve();
  }
}

// The class that ScopeTest delivers the in scope and out of scope paths in.
class ScopeTestPath {
  #parentHandle;
  #fileName;
  #relativePathComponents;

  constructor(parentHandle, parentRelativePathComponents) {
    this.#parentHandle = parentHandle;
    this.#fileName = getUniqueName();
    this.#relativePathComponents =
        [...parentRelativePathComponents, this.#fileName];
  }

  parentHandle() {
    return this.#parentHandle;
  }

  fileName() {
    return this.#fileName;
  }

  // Returns the relative path components to the watched directory.
  relativePathComponents() {
    return this.#relativePathComponents;
  }

  createHandle() {
    return this.#parentHandle.getFileHandle(this.#fileName, {create: true});
  }
}
