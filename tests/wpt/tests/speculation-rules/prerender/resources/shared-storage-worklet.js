class ReadOperation {
  async run() {
    return await this.sharedStorage.get("prerender");
  }
}
