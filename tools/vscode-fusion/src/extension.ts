import * as path from 'path';
import {
  workspace,
  ExtensionContext,
  commands,
  TextDocument,
  DiagnosticCollection,
  languages,
  window,
  StatusBarAlignment,
  StatusBarItem
} from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from 'vscode-languageclient/node';
import { createDiagnostics, DiagnosticManager } from './diagnostics';
import { createFormatter } from './formatter';
import { FusionCompletionProvider } from './completion';
import { FusionHoverProvider } from './hover';

let client: LanguageClient | undefined;
let diagnosticManager: DiagnosticManager | undefined;
let statusBarItem: StatusBarItem;

export function activate(context: ExtensionContext): void {
  statusBarItem = window.createStatusBarItem(StatusBarAlignment.Right, 100);
  statusBarItem.text = '$(beaker) Fusion';
  statusBarItem.tooltip = 'Fusion Language Server';
  statusBarItem.show();
  context.subscriptions.push(statusBarItem);

  diagnosticManager = createDiagnostics(context);
  context.subscriptions.push(diagnosticManager);

  registerProviders(context);
  registerCommands(context);
  setupLanguageClient(context);

  if (workspace.getConfiguration('fusion').get<boolean>('diagnosticsOnOpen')) {
    const editor = window.activeTextEditor;
    if (editor && editor.document.languageId === 'fusion') {
      diagnosticManager.diagnose(editor.document);
    }
  }

  context.subscriptions.push(
    window.onDidChangeActiveTextEditor((editor) => {
      if (editor && editor.document.languageId === 'fusion') {
        statusBarItem.text = '$(beaker) Fusion: Active';
      } else {
        statusBarItem.text = '$(beaker) Fusion';
      }
    })
  );
}

function registerProviders(context: ExtensionContext): void {
  const selector = { language: 'fusion', scheme: 'file' };

  context.subscriptions.push(
    languages.registerCompletionItemProvider(
      selector,
      new FusionCompletionProvider(),
      '.',
      ':',
      '(',
      '['
    )
  );

  context.subscriptions.push(
    languages.registerHoverProvider(selector, new FusionHoverProvider())
  );

  const formatter = createFormatter();
  context.subscriptions.push(
    languages.registerDocumentFormattingEditProvider(selector, formatter)
  );
  context.subscriptions.push(
    languages.registerDocumentRangeFormattingEditProvider(selector, formatter)
  );

  if (workspace.getConfiguration('fusion').get<boolean>('diagnosticsOnSave')) {
    context.subscriptions.push(
      workspace.onDidSaveTextDocument((doc) => {
        if (doc.languageId === 'fusion' && diagnosticManager) {
          diagnosticManager.diagnose(doc);
        }
      })
    );
  }
}

function runFucCommand(args: string[]): Thenable<string> {
  const config = workspace.getConfiguration('fusion');
  const fucPath = config.get<string>('fucPath', 'fuc');

  const { execFile } = require('child_process');
  return new Promise((resolve, reject) => {
    execFile(fucPath, args, { cwd: workspace.workspaceFolders?.[0]?.uri.fsPath }, (err: any, stdout: string, stderr: string) => {
      if (err) {
        reject(stderr || err.message);
      } else {
        resolve(stdout);
      }
    });
  });
}

function registerCommands(context: ExtensionContext): void {
  context.subscriptions.push(
    commands.registerCommand('fusion.restartServer', () => {
      if (client) {
        client.restart();
        window.showInformationMessage('Fusion language server restarted.');
      }
    })
  );

  context.subscriptions.push(
    commands.registerCommand('fusion.format', async () => {
      const editor = window.activeTextEditor;
      if (editor && editor.document.languageId === 'fusion') {
        await languages.executeDocumentFormattingEdits(
          editor.document,
          {},
          undefined
        );
      }
    })
  );

  context.subscriptions.push(
    commands.registerCommand('fusion.diagnose', () => {
      const editor = window.activeTextEditor;
      if (editor && editor.document.languageId === 'fusion' && diagnosticManager) {
        diagnosticManager.diagnose(editor.document);
        window.showInformationMessage('Fusion diagnostics run.');
      }
    })
  );

  context.subscriptions.push(
    commands.registerCommand('fusion.compile', async () => {
      const editor = window.activeTextEditor;
      if (editor && editor.document.languageId === 'fusion') {
        try {
          statusBarItem.text = '$(sync~spin) Fusion: Compiling...';
          const output = await runFucCommand(['compile', editor.document.fileName]);
          window.showInformationMessage(`Fusion: Compilation succeeded.\n${output}`);
        } catch (err) {
          window.showErrorMessage(`Fusion: Compilation failed.\n${err}`);
        } finally {
          statusBarItem.text = '$(beaker) Fusion';
        }
      }
    })
  );

  context.subscriptions.push(
    commands.registerCommand('fusion.run', async () => {
      const editor = window.activeTextEditor;
      if (editor && editor.document.languageId === 'fusion') {
        try {
          statusBarItem.text = '$(play~spin) Fusion: Running...';
          const output = await runFucCommand(['run', editor.document.fileName]);
          const panel = window.createOutputChannel('Fusion Output');
          panel.appendLine(output);
          panel.show();
        } catch (err) {
          window.showErrorMessage(`Fusion: Run failed.\n${err}`);
        } finally {
          statusBarItem.text = '$(beaker) Fusion';
        }
      }
    })
  );

  context.subscriptions.push(
    commands.registerCommand('fusion.test', async () => {
      const editor = window.activeTextEditor;
      if (editor && editor.document.languageId === 'fusion') {
        try {
          statusBarItem.text = '$(beaker) Fusion: Testing...';
          const output = await runFucCommand(['test', editor.document.fileName]);
          const panel = window.createOutputChannel('Fusion Test Results');
          panel.appendLine(output);
          panel.show();
        } catch (err) {
          window.showErrorMessage(`Fusion: Tests failed.\n${err}`);
        } finally {
          statusBarItem.text = '$(beaker) Fusion';
        }
      }
    })
  );

  context.subscriptions.push(
    commands.registerCommand('fusion.quantum.simulate', async () => {
      const editor = window.activeTextEditor;
      if (editor && editor.document.languageId === 'fusion') {
        try {
          statusBarItem.text = '$(beaker) Fusion: Simulating quantum circuit...';
          const output = await runFucCommand(['quantum', 'simulate', editor.document.fileName]);
          const panel = window.createOutputChannel('Fusion Quantum Simulation');
          panel.appendLine(output);
          panel.show();
        } catch (err) {
          window.showErrorMessage(`Fusion: Quantum simulation failed.\n${err}`);
        } finally {
          statusBarItem.text = '$(beaker) Fusion';
        }
      }
    })
  );

  context.subscriptions.push(
    commands.registerCommand('fusion.ai.chat', async () => {
      const input = await window.showInputBox({
        prompt: 'Enter prompt for Fusion AI (Cortex)',
        placeHolder: 'e.g., Optimize this function for parallel execution'
      });
      if (input) {
        try {
          statusBarItem.text = '$(beaker) Fusion: AI processing...';
          const output = await runFucCommand(['cortex', 'chat', input]);
          const panel = window.createOutputChannel('Fusion AI');
          panel.appendLine(output);
          panel.show();
        } catch (err) {
          window.showErrorMessage(`Fusion AI: Failed.\n${err}`);
        } finally {
          statusBarItem.text = '$(beaker) Fusion';
        }
      }
    })
  );

  context.subscriptions.push(
    commands.registerCommand('fusion.crypto.sign', async () => {
      const editor = window.activeTextEditor;
      if (editor && editor.document.languageId === 'fusion') {
        try {
          statusBarItem.text = '$(beaker) Fusion: Signing...';
          const output = await runFucCommand(['crypto', 'sign', editor.document.fileName]);
          window.showInformationMessage(`Fusion: Signed successfully.\n${output}`);
        } catch (err) {
          window.showErrorMessage(`Fusion: Signing failed.\n${err}`);
        } finally {
          statusBarItem.text = '$(beaker) Fusion';
        }
      }
    })
  );

  context.subscriptions.push(
    commands.registerCommand('fusion.deploy', async () => {
      const editor = window.activeTextEditor;
      if (editor && editor.document.languageId === 'fusion') {
        const target = await window.showQuickPick(
          ['kubernetes', 'faas', 'docker'],
          { placeHolder: 'Select deployment target' }
        );
        if (target) {
          try {
            statusBarItem.text = `$(cloud~spin) Fusion: Deploying to ${target}...`;
            const output = await runFucCommand(['deploy', '--target', target, editor.document.fileName]);
            window.showInformationMessage(`Fusion: Deployed to ${target}.\n${output}`);
          } catch (err) {
            window.showErrorMessage(`Fusion: Deployment failed.\n${err}`);
          } finally {
            statusBarItem.text = '$(beaker) Fusion';
          }
        }
      }
    })
  );
}

function setupLanguageClient(context: ExtensionContext): void {
  const config = workspace.getConfiguration('fusion');
  const fucPath = config.get<string>('fucPath', 'fuc');

  const serverModule = context.asAbsolutePath(
    path.join('out', 'server.js')
  );

  const serverOptions: ServerOptions = {
    run: {
      module: serverModule,
      transport: TransportKind.ipc
    },
    debug: {
      module: serverModule,
      transport: TransportKind.ipc,
      options: {
        execArgv: ['--nolazy', '--inspect=6009']
      }
    }
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'fusion' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.fus')
    }
  };

  client = new LanguageClient(
    'fusionLanguageServer',
    'Fusion Language Server',
    serverOptions,
    clientOptions
  );

  client.start().catch((err) => {
    console.error('Failed to start Fusion language server:', err);
    statusBarItem.text = '$(warning) Fusion: Server Error';
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
