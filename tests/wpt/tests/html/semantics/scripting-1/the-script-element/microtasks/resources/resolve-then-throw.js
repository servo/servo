self.log.push('importScripts()ed script');
Promise.resolve().then(() => self.log.push('promise'));
throw new Error('foo');
