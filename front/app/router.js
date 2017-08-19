import Ember from 'ember';
import config from './config/environment';

const Router = Ember.Router.extend({
  location: config.locationType,
  rootURL: config.rootURL
});

Router.map(function() {
  this.route('summary', { path: '/' });
  this.route('nightlies', { path: '/nightlies' });
  this.route('links', { path: '/links' });
  this.route('triage', { path: '/triage' });
  this.route('fcp_user', { path: '/fcp/:username' });
  this.route('fcp', { path: '/fcp' });
});

export default Router;
