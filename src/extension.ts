// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {

  // Hello World command
  const listFilesCommand = vscode.commands.registerCommand(
    'octavian.listWorkspaceFiles',
    async () => {

        vscode.window.showInformationMessage('Octavian command triggered.');

        if (!vscode.workspace.workspaceFolders) {
            vscode.window.showInformationMessage('No workspace open.');
            return;
        }

        const files = await vscode.workspace.findFiles('**/*');

        vscode.window.showInformationMessage(
            `Found ${files.length} files`
        );
    }
);

context.subscriptions.push(listFilesCommand);

}





// This method is called when your extension is deactivated
export function deactivate() {}
