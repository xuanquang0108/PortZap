// Sinh ui/dist/ từ source (index.html, app.js, input.css).
// KHÔNG sửa file trong dist/ — chạy `npm run build` (hoặc `npm run watch` khi dev).
import { execSync } from "node:child_process";
import { readFileSync, writeFileSync, mkdirSync, copyFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const dir = path.dirname(fileURLToPath(import.meta.url));
const dist = path.join(dir, "dist");
mkdirSync(dist, { recursive: true });

execSync("npx tailwindcss -i ./input.css -o ./dist/styles.css --minify", {
  cwd: dir,
  stdio: "inherit",
  shell: true,
});

const gen = (src, banner, out) => {
  const code = readFileSync(path.join(dir, src), "utf8");
  writeFileSync(path.join(dist, out), banner + code);
  console.log(`[build] dist/${out}`);
};

gen(
  "index.html",
  "<!-- GENERATED FILE — edit ui/index.html and run `npm run build` -->\n",
  "index.html"
);
gen(
  "app.js",
  "// GENERATED FILE — edit ui/app.js and run `npm run build`\n",
  "app.js"
);
const css = readFileSync(path.join(dist, "styles.css"), "utf8");
if (!css.startsWith("/* GENERATED")) {
  writeFileSync(
    path.join(dist, "styles.css"),
    "/* GENERATED FILE — edit ui/input.css and run `npm run build` */\n" + css
  );
}
console.log("[build] done");
