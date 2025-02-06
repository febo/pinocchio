import "zx/globals";

const args = process.argv.slice(2);

await Promise.all([
  $`tsx ./scripts/lint-clippy.mjs ${args}`.nothrow(),
  $`tsx ./scripts/lint-docs.mjs ${args}`.nothrow(),
  $`tsx ./scripts/lint-features.mjs ${args}`.nothrow(),
]);
