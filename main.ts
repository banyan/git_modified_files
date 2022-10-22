import { Command } from 'https://deno.land/x/cliffy@v0.25.2/command/mod.ts';

import {
  getModifiedFilesOnCommit,
  getModifiedFilesOnWorkingTree,
} from './src/git.ts';

async function main(object: string | undefined) {
  try {
    const files = object
      ? await getModifiedFilesOnCommit(object)
      : await getModifiedFilesOnWorkingTree();
    if (files) console.log(files);
  } catch (error) {
    console.error(error);
    Deno.exit(1);
  }
}

await new Command()
  .name('git_modified_files')
  .version('0.1.3')
  .description(
    'A Git subcommand to list modified files in git commit or from current working tree',
  )
  .arguments('[object:string]')
  .action(async (_, object) => {
    await main(object);
  })
  .parse();
