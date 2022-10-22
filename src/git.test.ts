import { assertEquals } from 'https://deno.land/std@0.160.0/testing/asserts.ts';
import { exec } from './utils.ts';
import {
  getModifiedFilesOnCommit,
  getModifiedFilesOnWorkingTree,
} from './git.ts';

const setupFirstCommit = async () => {
  const tmpTestDir = await Deno.makeTempDir();
  Deno.chdir(tmpTestDir);

  await exec('touch a.txt');
  await exec('git init --initial-branch=main');
  await exec('git config --local user.email "hi@example.com"');
  await exec('git config --local user.name "yo"');
  await exec('git add .');
  await exec('git commit -m Init');
};

Deno.test({
  name: 'getModifiedFilesOnWorkingTree',
  sanitizeResources: false,
  async fn(t) {
    await t.step(
      'it does not output modified files to stdout when there are no modified files',
      async () => {
        await setupFirstCommit();
        const files = await getModifiedFilesOnWorkingTree();
        assertEquals(files, '');
      },
    );

    await t.step(
      'it outputs modified files to stdout when there are modified files in working tree',
      async () => {
        await setupFirstCommit();
        await exec('touch b.txt');
        await exec('git add .');
        await exec('touch c.txt');
        const files = await getModifiedFilesOnWorkingTree();
        assertEquals(files, 'b.txt\nc.txt');
      },
    );

    await t.step(
      'it does not output modified files to stdout when there are deleted files in working tree',
      async () => {
        await setupFirstCommit();
        await exec('rm a.txt');
        const files = await getModifiedFilesOnWorkingTree();
        assertEquals(files, '');
      },
    );

    await t.step(
      'it does not output modified files to stdout when there deleted files in staging',
      async () => {
        await setupFirstCommit();
        await exec('rm a.txt');
        await exec('git add -u a.txt');
        const files = await getModifiedFilesOnWorkingTree();
        assertEquals(files, '');
      },
    );

    await t.step(
      'it does not output binary files to stdout when there are binary files in working tree',
      async () => {
        await setupFirstCommit();
        await exec('touch a.png');
        let files = await getModifiedFilesOnWorkingTree();
        await exec('git add a.png');
        files = await getModifiedFilesOnWorkingTree();
        assertEquals(files, '');
      },
    );

    await t.step(
      'it outputs renamed file to stdout when files are renamed',
      async () => {
        await setupFirstCommit();
        await exec('mv a.txt b.txt');
        let files = await getModifiedFilesOnWorkingTree();
        assertEquals(files, 'b.txt');
        await exec('git add -A');
        files = await getModifiedFilesOnWorkingTree();
        assertEquals(files, 'b.txt');
      },
    );

    await t.step(
      'it outputs relative path when directory is changed',
      async () => {
        await setupFirstCommit();
        await exec('mkdir foo');
        await exec('touch foo/a.txt');
        await exec('git add .');
        Deno.chdir('./foo');
        const files = await getModifiedFilesOnWorkingTree();
        assertEquals(files, 'a.txt');
      },
    );
  },
});

Deno.test({
  name: 'getModifiedFilesOnCommit',
  sanitizeResources: false,
  async fn(t) {
    await t.step('it outputs modified files to stdout', async () => {
      await setupFirstCommit();
      const object = await exec('git log --pretty=format:%h -n 1');
      const files = await getModifiedFilesOnCommit(object);
      assertEquals(files, 'a.txt');
    });

    await t.step(
      'it outputs modified files to stdout in case of merge commit',
      async () => {
        await setupFirstCommit();
        await exec('git checkout -b b-txt');
        await exec('touch b.txt');
        await exec('git add .');
        await exec('git commit -m b.txt');
        await exec('git checkout main');

        await exec('git checkout -b c-txt');
        await exec('touch c.txt');
        await exec('git add .');
        await exec('git commit -m c.txt');
        await exec('git checkout main');

        await exec('git merge --no-ff c-txt'); // Add no-ff option to imitate GitHub Pull Request
        await exec('git merge --no-ff b-txt');

        const object = await exec('git log --pretty=format:%h -n 1');
        const files = await getModifiedFilesOnCommit(object);
        assertEquals(files, 'b.txt');
      },
    );
  },
});
