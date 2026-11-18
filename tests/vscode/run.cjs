const fs = require('node:fs');
const path = require('node:path');
const { downloadAndUnzipVSCode, runTests, runVSCodeCommand } = require('@vscode/test-electron');

async function main() {
  const vsix = '../../pkgs/myriad-dreamin.tinymist-0.15.12-win32-x64.vsix';
  fs.accessSync(vsix, fs.constants.R_OK);

  const sandbox = fs.mkdtempSync('.profile-');
  const userData = path.join(sandbox, 'data');
  const extensions = path.join(sandbox, 'extensions');
  const options = { version: '1.141.0' };

  try {
    fs.cpSync('../../Code/snippets', path.join(userData, 'User', 'snippets'), { recursive: true });
    const vscodeExecutablePath = await downloadAndUnzipVSCode(options);
    await runVSCodeCommand([
      '--install-extension', vsix,
      '--user-data-dir', userData,
      '--extensions-dir', extensions,
    ], options);

    await runTests({
      vscodeExecutablePath,
      extensionDevelopmentPath: __dirname,
      extensionTestsPath: path.join(__dirname, 'suite.cjs'),
      launchArgs: [
        '--user-data-dir', path.resolve(userData),
        '--extensions-dir', path.resolve(extensions),
        '--skip-welcome',
        '--skip-release-notes',
      ],
    });
  } finally {
    fs.rmSync(sandbox, { recursive: true, force: true, maxRetries: 5 });
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
