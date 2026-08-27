'use strict';

const assert = require('assert');
const path = require('path');
const { readPromptState, route } = require('./extension');

function fakeIo(records) {
  return {
    readdirSync: () => Object.keys(records),
    readFileSync: (file) => records[path.basename(file)].json,
    statSync: (file) => ({ mtimeMs: records[path.basename(file)].mtime })
  };
}

(async () => {
  const records = {
    '20.json': { mtime: 2, json: JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10, 20], prompt_active: true }) },
    '30.json': { mtime: 1, json: JSON.stringify({ schema_version: 1, pid: 30, console_pids: [99], prompt_active: true }) }
  };
  assert.strictEqual(readPromptState('ignored', 10, fakeIo(records), () => true), true);
  records['20.json'].json = JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10, 20], prompt_active: false });
  assert.strictEqual(readPromptState('ignored', 10, fakeIo(records), () => true), false);
  assert.strictEqual(readPromptState('ignored', 99, fakeIo(records), () => false), false);
  assert.strictEqual(readPromptState('ignored', 88, fakeIo(records), () => true), false);

  const calls = [];
  const vscode = {
    window: { activeTerminal: { processId: Promise.resolve(10), sendText: (text, newline) => calls.push(['send', text, newline]) } },
    commands: { executeCommand: async (command) => calls.push(['fallback', command]) }
  };
  records['20.json'].json = JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10], prompt_active: true });
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
    await route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop');
    assert.deepStrictEqual(calls.pop(), ['send', '\u001b[1;5H', false]);
    records['20.json'].json = JSON.stringify({ schema_version: 1, pid: 20, console_pids: [10], prompt_active: false });
    await route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop');
    assert.deepStrictEqual(calls.pop(), ['fallback', 'workbench.action.terminal.scrollToTop']);
  } finally {
    original.readFileSync = oldRead;
    original.readdirSync = oldList;
    original.statSync = oldStat;
    process.kill = oldKill;
    if (oldLocal === undefined) delete process.env.LOCALAPPDATA; else process.env.LOCALAPPDATA = oldLocal;
  }
  console.log('VS Code bridge tests passed');
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});