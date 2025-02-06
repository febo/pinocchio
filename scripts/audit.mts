#!/usr/bin/env zx
import "zx/globals";

const advisories = [
  // ed25519-dalek: Double Public Key Signing Function Oracle Attack
  //
  // Remove once repo upgrades to ed25519-dalek v2
  "RUSTSEC-2022-0093",

  // curve25519-dalek
  //
  // Remove once repo upgrades to curve25519-dalek v4
  "RUSTSEC-2024-0344",

  // Crate:     tonic
  // Version:   0.9.2
  // Title:     Remotely exploitable Denial of Service in Tonic
  // Date:      2024-10-01
  // ID:        RUSTSEC-2024-0376
  // URL:       https://rustsec.org/advisories/RUSTSEC-2024-0376
  // Solution:  Upgrade to >=0.12.3
  "RUSTSEC-2024-0376",
];
const ignores: string[] = [];
advisories.forEach((x) => {
  ignores.push("--ignore");
  ignores.push(x);
});

await $`cargo audit ${ignores}`;
