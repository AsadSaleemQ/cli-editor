'use strict';

const assert = require('assert');
const path = require('path');
const manifest = require('./package.json');
const { activate, readCodexState, route, smartPaste } = require('./extension');

function fakeIo(records) {
  return {
    readdirSync: () => Object.keys(records),
    readFileSync: (file) => records[path.basename(file)].json,
    statSync: (file) => ({ mtimeMs: records[path.basename(file)].mtime })
  };
}

(async () => {
  assert.match(manifest.description, /enhanced Codex/);
  assert.ok(manifest.displayName.startsWith('Codex'));
  assert.deepStrictEqual(manifest.keywords.includes('codex'), true);

  const records = {
    '20.json': { mtime: 2, json: JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10, 20], prompt_active: true }) },
    '30.json': { mtime: 1, json: JSON.stringify({ schema_version: 1, pid: 30, console_pids: [99], prompt_active: true }) }
  };
  assert.deepStrictEqual(
    readCodexState('ignored', 10, fakeIo(records), () => true),
    { matched: true, active: true, mtime: 2 }
  );
  records['20.json'].json = JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10, 20], prompt_active: false });
  assert.deepStrictEqual(
    readCodexState('ignored', 10, fakeIo(records), () => true),
    { matched: true, active: false, mtime: 2 }
  );
  assert.deepStrictEqual(
    readCodexState('ignored', 88, fakeIo(records), () => true),
    { matched: false, active: false }
  );

  const calls = [];
  const vscode = {
    window: { activeTerminal: { processId: Promise.resolve(10), sendText: (text, newline) => calls.push(['send', text, newline]) } },
    commands: { executeCommand: async (...args) => calls.push(['command', ...args]) },
    env: { clipboard: { readText: async () => 'clipboard text' } }
  };
  const original = require('fs');
  const oldRead = original.readFileSync;
  const oldList = original.readdirSync;
  const oldStat = original.statSync;
  const oldKill = process.kill;
  const oldLocal = process.env.LOCALAPPDATA;
  try {
    original.readdirSync = fakeIo(records).readdirSync;
    original.readFileSync = fakeIo(records).readFileSync;
    original.statSync = fakeIo(records).statSync;
    process.kill = () => true;
    process.env.LOCALAPPDATA = 'ignored';

    records['20.json'].json = JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10], prompt_active: true });
    await route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop');
    assert.deepStrictEqual(calls.pop(), ['send', '\u001b[1;5H', false]);

    records['20.json'].json = JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10], prompt_active: false });
    await route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop');
    assert.deepStrictEqual(calls.pop(), ['command', 'workbench.action.terminal.scrollToTop']);

    await smartPaste(vscode);
    assert.deepStrictEqual(calls.pop(), ['command', 'workbench.action.terminal.paste']);

    vscode.env.clipboard.readText = async () => '';
    await smartPaste(vscode);
    assert.deepStrictEqual(calls.pop(), [
      'command',
      'workbench.action.terminal.sendSequence',
      { text: '\u0016' }
    ]);

    vscode.window.activeTerminal.processId = Promise.resolve(88);
    await smartPaste(vscode);
    assert.deepStrictEqual(calls.pop(), ['command', 'workbench.action.terminal.paste']);

    vscode.window.activeTerminal = undefined;
    await route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop');
    await smartPaste(vscode);
    assert.strictEqual(calls.length, 0);
  } finally {
    original.readFileSync = oldRead;
    original.readdirSync = oldList;
    original.statSync = oldStat;
    process.kill = oldKill;
    if (oldLocal === undefined) delete process.env.LOCALAPPDATA;
    else process.env.LOCALAPPDATA = oldLocal;
  }

  const registeredCommands = [];
  vscode.commands.registerCommand = (id) => {
    registeredCommands.push(id);
    return { dispose() {} };
  };
  const context = { subscriptions: [] };
  activate(context, vscode);
  const activationCommands = manifest.activationEvents.map((event) =>
    event.replace(/^onCommand:/, ''));
  const contributedCommands = manifest.contributes.commands.map(({ command }) => command);
  const keybindingCommands = manifest.contributes.keybindings.map(({ command }) => command);
  assert.deepStrictEqual(registeredCommands.sort(), activationCommands.sort());
  assert.deepStrictEqual(keybindingCommands.sort(), contributedCommands.sort());
  assert.ok(registeredCommands.includes('terminalSmartPaste.paste'));
  assert.ok(!contributedCommands.includes('terminalSmartPaste.paste'));
  console.log('VS Code bridge tests passed');
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
