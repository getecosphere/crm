const http = require("node:http");
const { readFile } = require("node:fs/promises");
const { readFileSync } = require("node:fs");
const path = require("node:path");

const root = __dirname;
const envFile = (() => { try { return Object.fromEntries(readFileSync(path.join(root, ".env"), "utf8").split(/\r?\n/).filter((line) => line && !line.startsWith("#")).map((line) => { const index = line.indexOf("="); return index < 0 ? [line, ""] : [line.slice(0, index), line.slice(index + 1)]; })); } catch { return {}; } })();
const port = Number(process.env.PORT || envFile.PORT);
const backendUrl = process.env.PUBLIC_API_URL || envFile.PUBLIC_API_URL;

if (!Number.isInteger(port) || port < 1 || port > 65535) {
  throw new Error("PORT is required. Run eco configure so Eco can assign this service port.");
}
if (!backendUrl) {
  throw new Error("PUBLIC_API_URL is required. Run eco configure so Eco can connect this frontend to its backend.");
}

const files = {
  "/": { file: "index.html", type: "text/html; charset=utf-8" },
  "/index.html": { file: "index.html", type: "text/html; charset=utf-8" },
  "/images/ecology-mark.webp": { file: "images/ecology-mark.webp", type: "image/webp" },
  "/runtime-config.js": { type: "application/javascript; charset=utf-8" }
};

http.createServer(async (request, response) => {
  const pathname = new URL(request.url, "http://" + (request.headers.host || "localhost")).pathname;
  const requested = files[pathname];
  if (!requested) {
    response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    response.end("Not found");
    return;
  }
  if (pathname === "/runtime-config.js") {
    response.writeHead(200, { "content-type": requested.type, "cache-control": "no-store" });
    response.end("window.__ECO_BACKEND_URL__ = " + JSON.stringify(backendUrl) + ";");
    return;
  }
  try {
    response.writeHead(200, { "content-type": requested.type, "cache-control": "no-cache" });
    response.end(await readFile(path.join(root, requested.file)));
  } catch {
    response.writeHead(500, { "content-type": "text/plain; charset=utf-8" });
    response.end("Starter asset is unavailable.");
  }
}).listen(port, "0.0.0.0", () => {
  console.log("Eco starter frontend listening on http://0.0.0.0:" + port);
});
