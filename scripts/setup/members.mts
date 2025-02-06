import 'zx/globals';
import { getCargo } from './shared.mts';

const members = `members=[${getCargo().workspace['members'].join(',')}]`;
await $`echo ${members} >> $GITHUB_OUTPUT`;
