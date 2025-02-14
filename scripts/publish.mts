#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  getCargo,
  popArgument,
  workingDirectory,
} from './setup/shared.mts';

const [folder, ...args] = cliArguments();
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

const fix = popArgument(args, '--dry-run');
const dryRun = argv['dry-run'] ?? false;

const [level] = args;
if (!level) {
  throw new Error('A version level — e.g. "patch" — must be provided.');
}

// Get the crate information.
const toml = getCargo(folder);
const crate = toml.package['name'];

// Go to the crate folder to release.
cd(path.dirname(manifestPath));

// Publish the new version.
const releaseArgs = dryRun
  ? []
  : ['--tag-name', `${crate}@v{{version}}`, '--no-confirm', '--execute'];
await $`cargo release ${level} ${releaseArgs}`;
