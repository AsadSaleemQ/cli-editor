'use strict';

const fs = require('fs');
const path = require('path');

function processIsAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (_) {
    return false;
  }
}

function readCodexState(directory, terminalPid, io = fs, isAlive = processIsAlive) {
  if (!Number.isInteger(terminalPid) || terminalPid <= 0) return { matched: false, active: false };
  let names;
  try {
    names = io.readdirSync(directory).filter((name) => /^\d+\.json$/.test(name));
  } catch (_) {
    return { matched: false, active: false };
  }
  const matches = [];
  for (const name of names) {
    const file = path.join(directory, name);
    try {
      const record = JSON.parse(io.readFileSync(file, 'utf8'));
      if (record.schema_version !== 1 || !isAlive(record.pid)) continue;
      if (!Array.isArray(record.console_pids) || !record.console_pids.includes(terminalPid)) continue;
      matches.push({ matched: true, active: record.prompt_active === true, mtime: io.statSync(file).mtimeMs });
    } catch (_) {
      // A writer may be replacing the record. Falling back is always safe.
    }
  }
  matches.sort((left, right) => right.mtime - left.mtime);
  return matches[0] || { matched: false, active: false };
}

async function activeCodexState(vscode) {
  const terminal = vscode.window.activeTerminal;
  if (!terminal) return { terminal: undefined, matched: false, active: false };
  const terminalPid = await terminal.processId;
  const localAppData = process.env.LOCALAPPDATA;
  const directory = localAppData && path.join(localAppData, 'CLIEditor', 'vscode-bridge');
  const state = directory
    ? readCodexState(directory, terminalPid)
    : { matched: false, active: false };
  return { terminal, ...state };
}

async function route(vscode, sequence, fallbackCommand) {
  const state = await activeCodexState(vscode);
  if (!state.terminal) return;
  if (state.active) {
    state.terminal.sendText(sequence, false);
    return;
  }
  await vscode.commands.executeCommand(fallbackCommand);
}

async function smartPaste(vscode) {
  const state = await activeCodexState(vscode);
  if (!state.terminal) return;
  if (!state.matched) {
    await vscode.commands.executeCommand('workbench.action.terminal.paste');
    return;
  }
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

function activate(context, vscode = require('vscode')) {
  context.subscriptions.push(
    vscode.commands.registerCommand('cliEditor.promptHome', () =>
      route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop')),
    vscode.commands.registerCommand('cliEditor.promptEnd', () =>
      route(vscode, '\u001b[1;5F', 'workbench.action.terminal.scrollToBottom')),
    vscode.commands.registerCommand('cliEditor.smartPaste', () =>
      smartPaste(vscode)),
    vscode.commands.registerCommand('terminalSmartPaste.paste', () =>
      smartPaste(vscode))
  );
}

function deactivate() {}

module.exports = { activate, deactivate, readCodexState, route, smartPaste };
