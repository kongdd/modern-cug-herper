const assert = require('node:assert/strict');
const vscode = require('vscode');

// A literal | in an expectation marks the cursor after insertion.
const cases = [
  ['\\doc', /^#import "@local\/modern-cug-report:[^"]+": \*\s+#show: .*header: "\|"\)/],
  ['\\int', /^integral \|$/],
  ['\\figure', /^#figure\(\s+image\("Figures", width: 70%\),\s+caption: \[\]\s+\) <fig_>\|$/],
  ['\\table', /^#figure\(\s+caption: \[\],\s+table\([\s\S]*\s\|\s+\),\s+\) <table_>$/],
  ['\\eq', /^#mitex\(`\$\$\r?\n\|\r?\n\$\$`\)$/],
  ['\\nonum', /^#nonum\(\[\$\s+\|\s+\$\]\)$/],
  ['\\e', /^#mi\("\|"\)$/],
  ['\\head', /^#h\(2em\)\|$/],
  ['\\frac', /^\\frac\{\|\}\{\}$/],
  ['\\omit', /^\/\/ <!-- omit in toc -->\|$/],
  ['\\cite', /^#cite\(<\|>, form: "prose"\)$/],
  ['\\highlight', /^#highlight\(\)\[\|\]$/],
];

exports.run = async () => {
  for (const [name, expected] of cases) {
    const document = await vscode.workspace.openTextDocument({ language: 'typst' });
    const editor = await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand('editor.action.insertSnippet', { name });

    const text = document.getText();
    const cursor = document.offsetAt(editor.selection.active);
    assert.match(text.slice(0, cursor) + '|' + text.slice(cursor), expected, name);
    console.log(`Passed: Typst ${name}`);
  }
};
