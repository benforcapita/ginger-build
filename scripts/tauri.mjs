import { spawnSync, spawn } from "node:child_process";
import { dirname, delimiter } from "node:path";

// Homebrew rustup may be available even when cargo/rustc are not on PATH.
const toolchain = spawnSync("rustup", ["which", "cargo"], { encoding: "utf8" });
const env = { ...process.env };
if (toolchain.status === 0) env.PATH = `${dirname(toolchain.stdout.trim())}${delimiter}${env.PATH ?? ""}`;
const child = spawn("tauri", process.argv.slice(2), { stdio: "inherit", env, shell: false });
child.on("error", (error) => { console.error(error.message); process.exitCode = 1; });
child.on("exit", (code) => { process.exitCode = code ?? 1; });
for (const signal of ["SIGINT", "SIGTERM"]) process.on(signal, () => child.kill(signal));
