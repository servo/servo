self.a = false;
importScripts('/content-security-policy/support/var-a.js');
postMessage({ 'executed': self.a });
