var Stash = function(inbound, outbound) {
  this.stashPath = '/presentation-api/controlling-ua/support/stash.py?id=';
  this.inbound = inbound;
  this.outbound = outbound;
}

// initialize a stash on wptserve
Stash.prototype.init = function() {
  return Promise.all([
    fetch(this.stashPath + this.inbound).then(response => {
      return response.text();
    }),
    fetch(this.stashPath + this.outbound).then(response => {
      return response.text();
    })
  ]);
}

// upload a test result to a stash on wptserve
Stash.prototype.send = function(result) {
  return fetch(this.stashPath + this.outbound, {
    method: 'POST',
    body: JSON.stringify({ type: 'data', data: result })
  }).then(response => {
    return response.text();
  }).then(text => {
    return text === 'ok' ? null : Promise.reject();
  })
};

// wait until a test result is uploaded to a stash on wptserve
Stash.prototype.receive = function() {
  return new Promise((resolve, reject) => {
    let intervalId;
    const interval = 500; // msec
    const polling = () => {
      return fetch(this.stashPath + this.inbound).then(response => {
        return response.text();
      }).then(text => {
        if (text) {
          try {
            const json = JSON.parse(text);
            if (json.type === 'data')
              resolve(json.data);
            else
              reject();
          } catch(e) {
            resolve(text);
          }
          clearInterval(intervalId);
        }
      });
    };
    intervalId = setInterval(polling, interval);
  });
};

// reset a stash on wptserve
Stash.prototype.stop = function() {
  return Promise.all([
    fetch(this.stashPath + this.inbound).then(response => {
      return response.text();
    }),
    fetch(this.stashPath + this.outbound).then(response => {
      return response.text();
    })
  ]).then(() => {
    return Promise.all([
      fetch(this.stashPath + this.inbound, {
        method: 'POST',
        body: JSON.stringify({ type: 'stop' })
      }).then(response => {
        return response.text();
      }),
      fetch(this.stashPath + this.outbound, {
        method: 'POST',
        body: JSON.stringify({ type: 'stop' })
      }).then(response => {
        return response.text();
      })
    ]);
  });
}
