import "zx/globals";

const args = process.argv.slice(2);

await Promise.all([
  $`tsx ./scripts/lint-clippy.mjs ${args}`,
  $`tsx ./scripts/lint-docs.mjs ${args}`,
  $`tsx ./scripts/lint-features.mjs ${args}`,
]);
