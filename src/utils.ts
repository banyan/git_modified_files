export async function exec(cmd: string) {
  const process = Deno.run({
    cmd: cmd.split(' '),
    stdout: 'piped',
    stderr: 'piped',
  });
  const { code } = await process.status();

  if (code === 0) {
    const rawOutput = await process.output();
    return new TextDecoder().decode(rawOutput).trim();
  } else {
    const rawError = await process.stderrOutput();
    const errorString = new TextDecoder().decode(rawError);
    throw new Error(errorString);
  }
}
