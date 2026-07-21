import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const candidates = process.argv.slice(2);
if (candidates.length === 0) {
  candidates.push("frontend/dist", "dist");
}

const roots = candidates.map((candidate) => resolve(process.cwd(), candidate)).filter(existsSync);
const generatedTodoPattern =
  /^[\t ]*\/\/ TODO we could test for more things here, like `Set`s and `Map`s\.\r?\n?/gm;

function jsFiles(root) {
  const entries = readdirSync(root);
  const files = [];
  for (const entry of entries) {
    const path = join(root, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      files.push(...jsFiles(path));
    } else if (path.endsWith(".js")) {
      files.push(path);
    }
  }
  return files;
}

for (const root of roots) {
  for (const file of jsFiles(root)) {
    const original = readFileSync(file, "utf8");
    const next = original.replace(generatedTodoPattern, "");
    if (next !== original) {
      writeFileSync(file, next);
    }
  }
}
