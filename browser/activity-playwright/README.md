# Playwright Browser Activities

Reusable `activity_exec` activities for driving a real Chromium browser via
[Playwright](https://playwright.dev/). A workflow starts a browser, runs
arbitrary JavaScript in the live page, and tears it down:

| FFQN | Signature | Returns |
|------|-----------|---------|
| `obelisk-browser:activity-playwright/browser.start` | `(container-name, socket, url)` | `record { container, image, socket, id, vncport: option<u16> }` |
| `obelisk-browser:activity-playwright/browser.eval` | `(socket, code)` | JSON-encoded page result (`string`) |
| `obelisk-browser:activity-playwright/browser.cleanup` | `(container-name, socket)` | `_` |

`start` runs the `docker.io/getobelisk/components-playwright` container, which
opens a Chromium page at `url` and listens on a Unix socket. `eval` sends the
`code` string to run in that page (as the body of an `async` function, so it may
`await` and should `return` a value) and returns the result. `cleanup` removes
the container and socket. Each returns `result<..., string>`.

The `code` runs with `page`, `context`, and `browser` (Playwright handles) in
scope. `eval`'s ok value is the page result **encoded as JSON text**; callers
`JSON.parse` it to recover the value.

## Host prerequisites

The activities shell out on the Obelisk host, which must have `docker`, `jq`,
and `socat` in `PATH`, and be able to pull the Playwright image.

## Building the image

The container image is separate from the activity components:

```sh
just build-image   # docker build -t docker.io/getobelisk/components-playwright:latest playwright
just push-image
```

## Running the activities

```sh
just serve   # obelisk server run --deployment obelisk-local.toml
```

In another terminal, drive the lifecycle:

```sh
sock=/tmp/pw/example.sock

just start example "$sock" https://example.com
just poke "$sock" 'return await page.title()'   # -> "\"Example Domain\""
just cleanup example "$sock"
```

A workflow should coordinate the same three calls and run `cleanup` in a
`finally` block so the container is always removed.

## Optional secret injection

To pass a token into page JavaScript without it appearing in params, argv, or the
execution result, declare a secret on the `eval` activity in your deployment:

```toml
[[activity_exec]]
ffqn = "obelisk-browser:activity-playwright/browser.eval"
secrets = ["token"]
# ...
```

Obelisk then writes `{ "secrets": { "token": "..." } }` to the script's stdin and
the page code can read `secrets.token`:

```js
await page.locator('#token').fill(secrets.token);
return await page.title();
```

## Headed mode / VNC

Set `HEADED=true` to run Chromium under Xvfb inside the container and expose VNC.
For a loopback `url` the container uses host networking and VNC listens on
`127.0.0.1:5900`; otherwise a dynamic localhost port is mapped and reported as
`vncport` in the `start` result. Set `IGNORE_HTTPS_ERRORS=true` only for a trusted
server with a self-signed certificate.
