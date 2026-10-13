import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

run("cargo", [
  "build",
  "-p",
  "ror2-profile-wasm",
  "--target",
  "wasm32-unknown-unknown",
  "--release",
  "--no-default-features",
  "--locked",
]);
run("wasm-bindgen", [
  "target/wasm32-unknown-unknown/release/ror2_profile_wasm.wasm",
  "--target",
  "web",
  "--out-dir",
  "wasm/pkg",
  "--out-name",
  "ror2_profile_wasm",
]);
