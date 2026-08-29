'use strict';

const assert = require('assert');
const path = require('path');
const { hasLiveSession, route } = require('./extension');

function fakeIo(records) {
  return {
    readdirSync: () => Object.keys(records),
    readFileSync: (file) => records[path.basename(file)].json
  };
}

(async () => {
  const records = {
    '20.json': { json: JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10, 20], prompt_active: true }) },
    '30.json': { json: JSON.stringify({ schema_version: 1, pid: 30, console_pids: [99], prompt_active: true }) }
  };
  assert.strictEqual(hasLiveSession('ignored', 10, fakeIo(records), () => true), true);
  records['20.json'].json = JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10, 20], prompt_active: false });
  assert.strictEqual(hasLiveSession('ignored', 10, fakeIo(records), () => true), true);
  assert.strictEqual(hasLiveSession('ignored', 99, fakeIo(records), () => false), false);
  assert.strictEqual(hasLiveSession('ignored', 88, fakeIo(records), () => true), false);

  const calls = [];
  const vscode = {
    window: { activeTerminal: { processId: Promise.resolve(10), sendText: (text, newline) => calls.push(['send', text, newline]) } },
    commands: { executeCommand: async (command) => calls.push(['fallback', command]) }
  };
  records['20.json'].json = JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10], prompt_active: true });
  const original = require('fs');
  const oldRead = original.readFileSync;
  const oldList = original.readdirSync;
  const oldKill = process.kill;
  const oldLocal = process.env.LOCALAPPDATA;
  try {
    original.readdirSync = fakeIo(records).readdirSync;
    original.readFileSync = fakeIo(records).readFileSync;
    process.kill = () => true;
    process.env.LOCALAPPDATA = 'ignored';
    await route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop');
    assert.deepStrictEqual(calls.pop(), ['send', '\u001b[1;5H', false]);
    await route(vscode, '\u001b[1;5F', 'workbench.action.terminal.scrollToBottom');
    assert.deepStrictEqual(calls.pop(), ['send', '\u001b[1;5F', false]);
    records['20.json'].json = JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10], prompt_active: false });
    await route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop');
    assert.deepStrictEqual(calls.pop(), ['send', '\u001b[1;5H', false]);
    process.kill = () => { throw new Error('not running'); };
    await route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop');
    assert.deepStrictEqual(calls.pop(), ['fallback', 'workbench.action.terminal.scrollToTop']);
    await route(vscode, '\u001b[1;5F', 'workbench.action.terminal.scrollToBottom');
    assert.deepStrictEqual(calls.pop(), ['fallback', 'workbench.action.terminal.scrollToBottom']);
  } finally {
    original.readFileSync = oldRead;
    original.readdirSync = oldList;
    process.kill = oldKill;
    if (oldLocal === undefined) delete process.env.LOCALAPPDATA; else process.env.LOCALAPPDATA = oldLocal;
  }
  console.log('VS Code bridge tests passed');
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
