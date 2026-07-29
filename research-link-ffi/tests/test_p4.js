// Design B for JS: two .node addons sharing one copy of the core crate.
const path = require('path');
const dir = path.join(__dirname, '..', 'out', 'p4');
function load(f) { const m = { exports: {} }; process.dlopen(m, path.join(dir, f)); return m.exports; }

const core = load('libp4_napi_core.so');
const ext = load('libp4_napi_ext.so');

const ep1 = new core.Endpoint(), ep2 = new core.Endpoint();
const p = new ext.Ping();
console.log('ep1.id =', ep1.id(), '| ep2.id =', ep2.id(), '| global @', ep1.globalAddr());
console.log('ping(ep1) ->', p.ping(ep1));
if (ep1.id() !== 0 || ep2.id() !== 1) throw new Error('core statics NOT shared');

// Type checks still hold across the addon boundary.
for (const [label, arg] of [['plain object', {}], ['wrong class', new ext.Ping()]]) {
  try { p.ping(arg); throw new Error(`${label} was accepted!`); }
  catch (e) { console.log(`ping(${label}) threw:`, e.message); }
}
// ...but the class identity does not.
console.log('core.Endpoint === ext.Endpoint :', core.Endpoint === ext.Endpoint);
console.log('OK: two .node addons, one shared copy of the core crate');
