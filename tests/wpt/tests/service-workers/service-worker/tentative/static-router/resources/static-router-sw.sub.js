var requests = [];
var errors = [];

const recordRequest = req => {
  requests.push({url: req.url, mode: req.mode});
};

const recordError = (error) => {
  errors.push(error);
};

const getRecords = () => {
  return {
    requests,
    errors
  };
}

const resetRecords = () => {
  requests = [];
  errors = [];
}

export {
  recordRequest,
  recordError,
  getRecords,
  resetRecords
};
