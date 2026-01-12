# Generic HTTP activity

Build and start the server:
```sh
just build serve
```

Issue a GET request:
```sh
obelisk client execution submit --follow \
  .../http.request -- \
  '"get"' \
  '"https://httpbin.org/get"' \
  '[]' \
  null
```

Issue a POST request:
```sh
obelisk client execution submit --follow \
  .../http.request \
  -- \
  '"post"' \
  '"https://httpbin.org/post"' \
  '[["Content-Type", "application/json"]]' \
  '{"text": "{\"hello\": \"world\"}"}'
```
