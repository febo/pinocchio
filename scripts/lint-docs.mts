#!/usr/bin/env zx
import "zx/globals";
import {
  cliArguments,
  getToolchainArgument,
  workingDirectory,
} from "./setup/shared.mts";

const [folder, ...args] = cliArguments();

const docArgs = ["--all-features", "--no-deps", ...args];

$.env["RUSTDOCFLAGS"] = "--cfg docsrs -D warnings";

const toolchain = getToolchainArgument("lint");
const manifestPath = path.join(workingDirectory, folder, "Cargo.toml");

await $`cargo ${toolchain} doc --manifest-path ${manifestPath} ${docArgs}`;
