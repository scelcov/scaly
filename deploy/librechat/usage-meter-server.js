"use strict";

const http = require("http");

const PORT = 8790;
const BALANCE_URL = "https://api.deepseek.com/user/balance";

function allowOrigin(origin) {
  if (!origin) {
    return false;
  }
  try {
    const url = new URL(origin);
    return url.port === "3080";
  } catch {
    return false;
  }
}

function send(res, status, body, origin) {
  const headers = {
    "Content-Type": "application/json; charset=utf-8",
    "Cache-Control": "no-store",
  };
  if (origin) {
    headers["Access-Control-Allow-Origin"] = origin;
    headers.Vary = "Origin";
  }
  res.writeHead(status, headers);
  res.end(JSON.stringify(body));
}

async function balance() {
  const key = process.env.DEEPSEEK_API_KEY;
  if (!key) {
    return { status: 503, body: { error: "unavailable" } };
  }
  const response = await fetch(BALANCE_URL, {
    headers: {
      Accept: "application/json",
      Authorization: `Bearer ${key}`,
    },
  });
  if (!response.ok) {
    return { status: 502, body: { error: "unavailable" } };
  }
  const payload = await response.json();
  const infos = Array.isArray(payload.balance_infos) ? payload.balance_infos : [];
  const info = infos.find((row) => row.currency === "USD") || infos[0];
  if (!info) {
    return { status: 502, body: { error: "unavailable" } };
  }
  return {
    status: 200,
    body: {
      available: payload.is_available === true,
      currency: info.currency,
      total: info.total_balance,
      granted: info.granted_balance,
      toppedUp: info.topped_up_balance,
    },
  };
}

const server = http.createServer((req, res) => {
  const origin = allowOrigin(req.headers.origin) ? req.headers.origin : "";
  if (req.method === "OPTIONS") {
    if (!origin) {
      res.writeHead(403);
      res.end();
      return;
    }
    res.writeHead(204, {
      "Access-Control-Allow-Origin": origin,
      "Access-Control-Allow-Methods": "GET",
      Vary: "Origin",
    });
    res.end();
    return;
  }
  if (req.method !== "GET" || req.url !== "/balance") {
    send(res, 404, { error: "not_found" }, origin);
    return;
  }
  if (!origin) {
    send(res, 403, { error: "forbidden" }, "");
    return;
  }
  balance()
    .then((result) => send(res, result.status, result.body, origin))
    .catch(() => send(res, 502, { error: "unavailable" }, origin));
});

server.listen(PORT, "0.0.0.0");
