#!/usr/bin/env node

import fs from "node:fs";
import net from "node:net";
import process from "node:process";
import { chromium } from "playwright";

const PAGE_TIMEOUT_MS = 30000;

function parseJsonArg(index, name) {
  const raw = process.argv[index];
  if (raw === undefined) throw new Error(`missing argument: ${name}`);
  try {
    return JSON.parse(raw);
  } catch {
    return raw;
  }
}

async function main() {
  const socketPath = parseJsonArg(2, "socket");
  const url = parseJsonArg(3, "url");
  const browser = await chromium.launch({ headless: process.env.HEADED !== "true" });
  const context = await browser.newContext({
    ignoreHTTPSErrors: process.env.IGNORE_HTTPS_ERRORS === "true",
  });
  const page = await context.newPage();
  page.setDefaultTimeout(PAGE_TIMEOUT_MS);
  page.setDefaultNavigationTimeout(PAGE_TIMEOUT_MS);
  await page.goto(url, { waitUntil: "domcontentloaded" });

  Object.assign(global, { browser, context, page });
  try {
    fs.unlinkSync(socketPath);
  } catch (_) {}

  const server = net.createServer({ allowHalfOpen: true }, (socket) => {
    let input = "";
    let handled = false;
    socket.setEncoding("utf8");
    const handleRequest = async (raw) => {
      if (handled) return;
      handled = true;
      let payload;
      try {
        const request = JSON.parse(raw);
        const secrets = request.secrets ?? {};
        const result = await eval(`(async () => { ${request.code} })()`);
        payload = { ok: true, result: result ?? null };
      } catch (error) {
        payload = { ok: false, error: error.message };
      }
      socket.write(`${JSON.stringify(payload)}\n`, () => socket.destroy());
    };
    socket.on("data", (chunk) => {
      input += chunk;
      const newline = input.indexOf("\n");
      if (newline !== -1) handleRequest(input.slice(0, newline));
    });
    socket.on("end", () => handleRequest(input));
  });

  const shutdown = async () => {
    server.close();
    try {
      fs.unlinkSync(socketPath);
    } catch (_) {}
    await browser.close();
    process.exit(0);
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);

  server.listen(socketPath, () => {
    fs.chmodSync(socketPath, 0o600);
    console.log(`Playwright server ready on ${socketPath}`);
  });
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
