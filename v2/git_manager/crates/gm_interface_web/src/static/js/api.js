/* Typed API client for JavaScript components */
const API = {
  base: '/api/v1',
  async get(path)        { const r = await fetch(this.base + path); return r.json(); },
  async post(path, body) { const r = await fetch(this.base + path, { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify(body) }); return r.json(); },
  async del(path)        { return fetch(this.base + path, { method: 'DELETE' }); },
  accounts: {
    list:       ()    => API.get('/accounts'),
    create:     (d)   => API.post('/accounts', d),
    remove:     (id)  => API.del(`/accounts/${id}`),
    setDefault: (id)  => API.post(`/accounts/${id}/default`, {}),
  },
  ssh: {
    list:     ()    => API.get('/ssh-keys'),
    generate: (d)   => API.post('/ssh-keys/generate', d),
    test:     (id)  => API.post(`/ssh-keys/${id}/test`, {}),
  },
  repos: {
    list:  () => API.get('/repositories'),
    clone: (d) => API.post('/repositories/clone', d),
  },
};
