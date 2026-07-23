import { existsSync } from "node:fs";
import { resolve, join, relative } from "node:path";

function parseRoot(argv) {
  const rootIndex = argv.indexOf("--root");
  if (rootIndex === -1) {
    return process.cwd();
  }

  const root = argv[rootIndex + 1];
  if (!root) {
    throw new Error("--root requires a path");
  }

  return root;
}

function expectedBinaries(root) {
  const dir = join(root, "src-tauri", "resources", "ffmpeg");
  return [join(dir, "ffmpeg.exe"), join(dir, "ffprobe.exe")];
}

try {
  const root = resolve(parseRoot(process.argv.slice(2)));
  const missing = expectedBinaries(root).filter((path) => !existsSync(path));

  if (missing.length > 0) {
    console.error("Missing FFmpeg local runtime files:");
    missing.forEach((path) => console.error(`- ${relative(root, path)}`));
    console.error("");
    console.error("Place ffmpeg.exe and ffprobe.exe under src-tauri/resources/ffmpeg.");
    console.error("For packaged runtime preparation, run src-tauri/scripts/prepare_runtime.ps1.");
    process.exit(1);
  }

  console.log("FFmpeg local runtime is ready: src-tauri/resources/ffmpeg");
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
