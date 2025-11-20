// Lightweight project scanner for duplicate imports and function names
// Scans JS/TS/TSX/JSX and Rust files. Outputs JSON report.

import { promises as fs } from "node:fs";
import path from "node:path";

const exts = new Set([".js", ".ts", ".tsx", ".jsx", ".rs"]);

async function* walk(dir) {
  for (const dirent of await fs.readdir(dir, { withFileTypes: true })) {
    const fp = path.join(dir, dirent.name);
    if (dirent.isDirectory()) {
      // skip some folders
      const base = dirent.name.toLowerCase();
      if (["node_modules", "target", "dist", "build", ".git"].includes(base))
        continue;
      yield* walk(fp);
    } else {
      yield fp;
    }
  }
}

function scanJsTs(content) {
  const issues = { duplicateImports: [], duplicateFunctions: [] };
  const importRe1 = /^\s*import\s+.+?from\s+['"]([^'"]+)['"];?/gm;
  const importRe2 = /^\s*import\s+['"]([^'"]+)['"];?/gm;
  const fnDeclRe = /^\s*function\s+([A-Za-z0-9_]+)\s*\(/gm;
  const arrowFnRe = /^\s*(?:const|let|var)\s+([A-Za-z0-9_]+)\s*=\s*\(/gm;

  const modules = {};
  let m;
  while ((m = importRe1.exec(content)))
    modules[m[1]] = (modules[m[1]] || 0) + 1;
  while ((m = importRe2.exec(content)))
    modules[m[1]] = (modules[m[1]] || 0) + 1;

  for (const [mod, count] of Object.entries(modules)) {
    if (count > 1) issues.duplicateImports.push({ module: mod, count });
  }

  const names = {};
  while ((m = fnDeclRe.exec(content))) names[m[1]] = (names[m[1]] || 0) + 1;
  while ((m = arrowFnRe.exec(content))) names[m[1]] = (names[m[1]] || 0) + 1;
  for (const [name, count] of Object.entries(names)) {
    if (count > 1) issues.duplicateFunctions.push({ name, count });
  }
  return issues;
}

function scanRust(content) {
  const issues = { duplicateImports: [], duplicateFunctions: [] };
  const useRe = /^\s*use\s+([^;]+);/gm;
  const fnRe = /^\s*pub?\s*fn\s+([A-Za-z0-9_]+)\s*\(/gm;
  const modules = {};
  let m;
  while ((m = useRe.exec(content)))
    modules[m[1].trim()] = (modules[m[1].trim()] || 0) + 1;
  for (const [mod, count] of Object.entries(modules)) {
    if (count > 1) issues.duplicateImports.push({ module: mod, count });
  }
  const names = {};
  while ((m = fnRe.exec(content))) names[m[1]] = (names[m[1]] || 0) + 1;
  for (const [name, count] of Object.entries(names)) {
    if (count > 1) issues.duplicateFunctions.push({ name, count });
  }
  return issues;
}

async function main() {
  const root = process.argv[2] ? path.resolve(process.argv[2]) : process.cwd();
  const results = { root, files: [] };

  for await (const fp of walk(root)) {
    const ext = path.extname(fp).toLowerCase();
    if (!exts.has(ext)) continue;
    const rel = path.relative(root, fp);
    const content = await fs.readFile(fp, "utf8");
    let issues;
    if (ext === ".rs") issues = scanRust(content);
    else issues = scanJsTs(content);
    if (issues.duplicateImports.length || issues.duplicateFunctions.length) {
      results.files.push({ file: rel.replace(/\\/g, "/"), ...issues });
    }
  }

  const outDir = path.join(root, "reports");
  await fs.mkdir(outDir, { recursive: true });
  const outPath = path.join(outDir, "imports-variables-duplicates.json");
  await fs.writeFile(outPath, JSON.stringify(results, null, 2), "utf8");
  console.log(`[scan-imports] Report -> ${path.relative(root, outPath)}`);
  if (results.files.length === 0)
    console.log("[scan-imports] No duplicates found.");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
