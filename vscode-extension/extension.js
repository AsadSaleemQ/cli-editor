'use strict';

function route(vscode, sequence) {
  const terminal = vscode.window.activeTerminal;
  if (!terminal) return;
  terminal.sendText(sequence, false);
}

async function smartPaste(vscode) {
  const text = await vscode.env.clipboard.readText();
  if (text.length > 0) {
    await vscode.commands.executeCommand('workbench.action.terminal.paste');
    return;
  }

  await vscode.commands.executeCommand(
    'workbench.action.terminal.sendSequence',
    { text: '\u0016' }
  );
}

function activate(context) {
  const vscode = require('vscode');
  context.subscriptions.push(
    vscode.commands.registerCommand('cliEditor.promptHome', () =>
      route(vscode, '\u001b[1;5H')),
    vscode.commands.registerCommand('cliEditor.promptEnd', () =>
      route(vscode, '\u001b[1;5F')),
    vscode.commands.registerCommand('cliEditor.smartPaste', () =>
      smartPaste(vscode)),
    vscode.commands.registerCommand('terminalSmartPaste.paste', () =>
      smartPaste(vscode))
  );
}

function deactivate() {}

module.exports = { activate, deactivate, route, smartPaste };
