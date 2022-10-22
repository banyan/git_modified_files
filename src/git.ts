import { exec } from './utils.ts';

export async function getModifiedFilesOnWorkingTree() {
  return (await exec('git status --short'))
    .split('\n')
    .filter((line) => line.charAt(0) !== 'D' && line.charAt(1) !== 'D')
    .filter(
      (line) =>
        !/\.(jpe?g|png|webp|avif|gif|svg|eot|mp3|ttf|wav|wof)$/i.test(line),
    )
    .map((line) => {
      return line.charAt(0) === 'R'
        ? line.split(' ').at(-1)
        : line.split(' ').filter(Boolean).slice(1);
    })
    .join('\n');
}
export async function getModifiedFilesOnCommit(object: string) {
  const output = (await exec(`git show --summary --format=%P ${object}`));
  const objects = output.split('\n')[0].split(' ');
  return objects.length === 2
    ? await exec(`git diff --name-only ${objects[0]}...${objects[1]}`)
    : await exec(`git log -m -1 --name-only --pretty=format: ${object}`);
}
