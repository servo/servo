class WriteOperation {
  async run() {
    return this.sharedStorage.set("prerender", true);
  }
}

class SelectURLOperation {
  async run() {
    return 0;
  }
}

register('test-prerender', WriteOperation);
register('test-prerender-selection', SelectURLOperation);
