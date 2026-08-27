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

function readPromptState(directory, terminalPid, io = fs, isAlive = processIsAlive) {
  if (!Number.isInteger(terminalPid) || terminalPid <= 0) return false;
  let names;
  try {
    names = io.readdirSync(directory).filter((name) => /^\d+\.json$/.test(name));
  } catch (_) {
    return false;
  }
  const matches = [];
  for (const name of names) {
    const file = path.join(directory, name);
    try {
      const record = JSON.parse(io.readFileSync(file, 'utf8'));
      if (record.schema_version !== 1 || !isAlive(record.pid)) continue;
      if (!Array.isArray(record.console_pids) || !record.console_pids.includes(terminalPid)) continue;
      matches.push({ active: record.prompt_active === true, mtime: io.statSync(file).mtimeMs });
    } catch (_) {
      // A writer may be replacing the tiny record. Falling back is always safe.
    }
  }
  matches.sort((left, right) => right.mtime - left.mtime);
  return matches.length > 0 && matches[0].active;
}

async function route(vscode, sequence, fallbackCommand) {
  const terminal = vscode.window.activeTerminal;
  if (!terminal) return;
  const terminalPid = await terminal.processId;
  const localAppData = process.env.LOCALAPPDATA;
  const directory = localAppData && path.join(localAppData, 'CLIEditor', 'vscode-bridge');
  if (directory && readPromptState(directory, terminalPid)) {
    terminal.sendText(sequence, false);
    return;
  }
  await vscode.commands.executeCommand(fallbackCommand);
}

function activate(context) {
  const vscode = require('vscode');
  context.subscriptions.push(
    vscode.commands.registerCommand('cliEditor.promptHome', () =>
      route(vscode, '\u001b[1;5H', 'workbench.action.terminal.scrollToTop')),
    vscode.commands.registerCommand('cliEditor.promptEnd', () =>
      route(vscode, '\u001b[1;5F', 'workbench.action.terminal.scrollToBottom'))
  );
}

function deactivate() {}

module.exports = { activate, deactivate, readPromptState, route };