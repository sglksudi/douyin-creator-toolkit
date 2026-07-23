import { mkdirSync, writeFileSync } from "node:fs";
import { rm, mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import assert from "node:assert/strict";

const script = join(process.cwd(), "scripts", "doctor-ffmpeg.mjs");

async function withTempRepo(testFn) {
  const root = await mkdtemp(join(tmpdir(), "ffmpeg-doctor-"));
  try {
    await testFn(root);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

function runDoctor(root) {
  return spawnSync(process.execPath, [script, "--root", root], {
    cwd: process.cwd(),
    encoding: "utf8",
  });
}

function createBinary(root, name) {
  const dir = join(root, "src-tauri", "resources", "ffmpeg");
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, name), "placeholder");
}

await withTempRepo(async (root) => {
  createBinary(root, "ffmpeg.exe");
  createBinary(root, "ffprobe.exe");

  const result = runDoctor(root);

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /FFmpeg local runtime is ready/);
});

await withTempRepo(async (root) => {
  createBinary(root, "ffmpeg.exe");

  const result = runDoctor(root);

  const output = `${result.stdout}\n${result.stderr}`;

  assert.equal(result.status, 1, output);
  assert.match(output, /Missing FFmpeg local runtime files/);
  assert.match(output, /src-tauri[\\/]resources[\\/]ffmpeg[\\/]ffprobe\.exe/);
});

console.log("FFmpeg doctor tests passed");
