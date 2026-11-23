import {profile} from '{{GET[import]}}';

try {
  self.postMessage(await profile());
} catch (error) {
  self.postMessage({success: false, error: error.name, message: error.message});
}
