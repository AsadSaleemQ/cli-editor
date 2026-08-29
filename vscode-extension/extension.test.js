'use strict';

const assert = require('assert');
const { route, smartPaste } = require('./extension');

(async () => {
  const calls = [];
  const vscode = {
    window: { activeTerminal: { sendText: (text, newline) => calls.push(['send', text, newline]) } }
  };
  route(vscode, '\u001b[1;5H');
  assert.deepStrictEqual(calls.pop(), ['send', '\u001b[1;5H', false]);
  route(vscode, '\u001b[1;5F');
  assert.deepStrictEqual(calls.pop(), ['send', '\u001b[1;5F', false]);

  vscode.window.activeTerminal = undefined;
  assert.doesNotThrow(() => route(vscode, '\u001b[1;5H'));

  const commandCalls = [];
  vscode.commands = {
    executeCommand: async (...args) => commandCalls.push(args)
  };
  vscode.env = { clipboard: { readText: async () => 'clipboard text' } };
  await smartPaste(vscode);
  assert.deepStrictEqual(commandCalls.pop(), ['workbench.action.terminal.paste']);

  vscode.env.clipboard.readText = async () => '';
  await smartPaste(vscode);
  assert.deepStrictEqual(commandCalls.pop(), [
    'workbench.action.terminal.sendSequence',
    { text: '\u0016' }
  ]);
  console.log('VS Code bridge tests passed');
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
