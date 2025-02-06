import 'zx/globals';
import { getCargo } from './shared.mts';

const members = getCargo().workspace['members'] as string[];
const membersAsJson = JSON.stringify(members);

await $`echo members=${membersAsJson} >> $GITHUB_OUTPUT`;
