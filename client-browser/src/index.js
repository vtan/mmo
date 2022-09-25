const rust = import('../../client/pkg');

rust
  .then(m => m.start())
  .catch(console.error);
