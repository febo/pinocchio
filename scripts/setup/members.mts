import 'zx/globals';
import { getCargo } from './shared.mts';

const members = getCargo().workspace['members'];

await $`echo "members=[${members.join(',')}]" >> $GITHUB_OUTPUT`;
