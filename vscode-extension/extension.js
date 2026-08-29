'use strict';

function route(vscode, sequence) {
  const terminal = vscode.window.activeTerminal;
  if (!terminal) return;
  terminal.sendText(sequence, false);
}

function activate(context) {
  const vscode = require('vscode');
  context.subscriptions.push(
    vscode.commands.registerCommand('cliEditor.promptHome', () =>
      route(vscode, '\u001b[1;5H')),
    vscode.commands.registerCommand('cliEditor.promptEnd', () =>
      route(vscode, '\u001b[1;5F'))
  );
}

function deactivate() {}

module.exports = { activate, deactivate, route };
