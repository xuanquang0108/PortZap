// Dev watch: tailwind --watch + tự copy index.html/app.js sang dist khi đổi.
// Chạy: npm run watch  (Ctrl+C để dừng)
import { spawn } from "node:child_process";
import { watch, copyFileSync, mkdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const dir = path.dirname(fileURLToPath(import.meta.url));
const dist = path.join(dir, "dist");
mkdirSync(dist, { recursive: true });

const copy = (f) => {
  copyFileSync(path.join(dir, f), path.join(dist, f));
  console.log(`[watch] copied ${f}`);
};

for (const f of ["index.html", "app.js"]) copy(f);

const tw = spawn(
  "npx",
  ["tailwindcss", "-i", "./input.css", "-o", "./dist/styles.css", "--watch"],
  { cwd: dir, stdio: "inherit", shell: true }
);

for (const f of ["index.html", "app.js"]) {
  watch(path.join(dir, f), () => copy(f));
}

console.log("[watch] watching ui/ (css + html + js)...");
process.on("SIGINT", () => {
  tw.kill();
  process.exit(0);
});
