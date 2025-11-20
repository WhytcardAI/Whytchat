import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// script path: <repo>/scripts/analysis/validate-structure.mjs
const projectRoot = path.resolve(__dirname, "..", "..");
const FRONTEND_SRC = path.join(projectRoot, "Frontend", "src");
const COMPONENTS_DIR = path.join(FRONTEND_SRC, "components");

function listSourceFiles(dir) {
  const results = [];
  if (!fs.existsSync(dir)) return results;
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const ent of entries) {
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) {
      results.push(...listSourceFiles(full));
    } else if (/\.(jsx|tsx|js|ts)$/.test(ent.name)) {
      results.push(full);
    }
  }
  return results;
}

function readLines(text) {
  return text.split(/\r?\n/);
}

function validate() {
  const errors = [];
  const warnings = [];

  if (!fs.existsSync(FRONTEND_SRC)) {
    errors.push(
      "ERROR: 'Frontend/src' is missing. Make sure the Frontend exists."
    );
  } else {
    if (!fs.existsSync(COMPONENTS_DIR)) {
      warnings.push(
        "WARN: 'src/components' directory is missing. Consider creating it."
      );
    } else {
      const entries = fs.readdirSync(COMPONENTS_DIR, { withFileTypes: true });
      for (const ent of entries) {
        const full = path.join(COMPONENTS_DIR, ent.name);
        if (ent.isDirectory()) {
          const idxCandidates = [
            "index.jsx",
            "index.tsx",
            "index.js",
            "index.ts",
          ];
          const hasIndex = idxCandidates.some((c) =>
            fs.existsSync(path.join(full, c))
          );
          if (!hasIndex) {
            errors.push(
              `ERROR: component folder '${ent.name}' missing index.(jsx|tsx|js|ts)`
            );
          }
        } else if (/\.(jsx|tsx|js|ts)$/.test(ent.name)) {
          errors.push(
            `ERROR: Found component file directly under components/: ${ent.name}`
          );
        }
      }
    }

    // Scan all source files
    const srcFiles = listSourceFiles(FRONTEND_SRC);

    // Large file detection
    for (const f of srcFiles) {
      try {
        const stat = fs.statSync(f);
        if (stat.size > 150 * 1024) {
          warnings.push(
            `WARNING: large file '${path.relative(
              FRONTEND_SRC,
              f
            )}' (${Math.round(stat.size / 1024)} KB)`
          );
        }
      } catch (e) {}
    }

    // Detect component definitions outside components/*
    const detectCompDefs = [];
    const compFuncRegex =
      /(^|\n)\s*(?:export\s+)?function\s+([A-Z][A-Za-z0-9_]*)\s*\(/g;
    const compConstRegex =
      /(^|\n)\s*(?:export\s+)?(?:const|let|var)\s+([A-Z][A-Za-z0-9_]*)\s*=\s*(?:\([^\)]*\)\s*=>|[^=>]*=>)/g;

    for (const f of srcFiles) {
      const rel = path.relative(FRONTEND_SRC, f);
      if (rel.startsWith("components" + path.sep)) continue; // skip real components
      let text = "";
      try {
        text = fs.readFileSync(f, "utf8");
      } catch (e) {
        continue;
      }

      // Quick JSX heuristic: if no '<' present, probably not a component file
      if (!text.includes("<")) continue;

      let m;
      const found = [];
      compFuncRegex.lastIndex = 0;
      while ((m = compFuncRegex.exec(text)) !== null) {
        found.push({ name: m[2], index: m.index });
      }
      compConstRegex.lastIndex = 0;
      while ((m = compConstRegex.exec(text)) !== null) {
        found.push({ name: m[2], index: m.index });
      }

      if (found.length > 0) {
        // report as ERROR: components must live under src/components
        const names = found
          .map((x) => x.name)
          .slice(0, 5)
          .join(", ");
        errors.push(
          `ERROR: Component(s) [${names}] defined outside 'src/components' in '${rel}'.`
        );
      }
    }
  }

  // Print results
  if (errors.length === 0 && warnings.length === 0) {
    console.log("OK: structure valid");
    process.exit(0);
  }

  for (const w of warnings) console.log(w);
  for (const e of errors) console.log(e);

  if (errors.length > 0) process.exit(2);
  process.exit(0);
}

if (import.meta.url === `file://${process.argv[1]}`) validate();
