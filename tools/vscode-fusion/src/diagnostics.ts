import {
  Diagnostic,
  DiagnosticCollection,
  DiagnosticSeverity,
  Range,
  Position,
  TextDocument,
  workspace,
  languages,
  Disposable,
  Uri
} from 'vscode';
import { execFile } from 'child_process';
import { promisify } from 'util';

const execFileAsync = promisify(execFile);

export interface DiagnosticManager extends Disposable {
  diagnose(document: TextDocument): Promise<void>;
  clear(uri: Uri): void;
  clearAll(): void;
}

export function createDiagnostics(context: { subscriptions: { push: (d: Disposable) => void } }): DiagnosticManager {
  const collection: DiagnosticCollection = languages.createDiagnosticCollection('fusion');

  async function diagnose(document: TextDocument): Promise<void> {
    const config = workspace.getConfiguration('fusion');
    const fucPath = config.get<string>('fucPath', 'fuc');

    clear(document.uri);

    try {
      const diagnostics = await runFucCheck(document, fucPath);
      collection.set(document.uri, diagnostics);
    } catch (error) {
      const diagnostics = parseCompileErrors(document, String(error));
      collection.set(document.uri, diagnostics);
    }
  }

  function clear(uri: Uri): void {
    collection.delete(uri);
  }

  function clearAll(): void {
    collection.clear();
  }

  context.subscriptions.push(collection);

  return { diagnose, clear, clearAll, dispose: () => collection.dispose() };
}

async function runFucCheck(document: TextDocument, fucPath: string): Promise<Diagnostic[]> {
  const documentText = document.getText();

  const args = ['check', '--json', '--stdin'];
  const options = {
    timeout: 10000,
    maxBuffer: 1024 * 1024
  };

  try {
    const { stdout, stderr } = await execFileAsync(fucPath, args, {
      ...options,
      input: documentText
    });

    if (stdout) {
      return parseFucOutput(document, stdout);
    }

    if (stderr) {
      return parseCompileErrors(document, stderr);
    }

    return [];
  } catch (error: any) {
    if (error.stdout) {
      return parseFucOutput(document, error.stdout);
    }
    if (error.stderr) {
      return parseCompileErrors(document, error.stderr);
    }
    return [];
  }
}

function parseFucOutput(document: TextDocument, output: string): Diagnostic[] {
  const diagnostics: Diagnostic[] = [];

  try {
    const json = JSON.parse(output);
    if (Array.isArray(json)) {
      for (const item of json) {
        const diagnostic = parseFucDiagnostic(document, item);
        if (diagnostic) {
          diagnostics.push(diagnostic);
        }
      }
    }
  } catch {
    return parseTextOutput(document, output);
  }

  return diagnostics;
}

function parseFucDiagnostic(document: TextDocument, item: any): Diagnostic | null {
  const severity = mapSeverity(item.severity || item.level);
  const range = parseRange(document, item.range || item.span);
  const message = item.message || item.error || 'Unknown error';
  const code = item.code || item.errorCode;
  const source = 'fuc';

  const diagnostic = new Diagnostic(range, message, severity);
  diagnostic.source = source;

  if (code !== undefined) {
    diagnostic.code = String(code);
  }

  if (item.relatedInformation) {
    const related: Diagnostic[] = [];
    for (const info of item.relatedInformation) {
      const relatedRange = parseRange(document, info.range || info.span);
      const relatedDiag = new Diagnostic(relatedRange, info.message || '', severity);
      relatedDiag.source = source;
      related.push(relatedDiag);
    }
  }

  if (item.tags) {
    diagnostic.tags = item.tags;
  }

  if (item.codeActions && Array.isArray(item.codeActions)) {
    // Code actions would be provided through the language server
  }

  return diagnostic;
}

function parseRange(document: TextDocument, range: any): Range {
  if (!range) {
    return new Range(0, 0, 0, 0);
  }

  const startLine = (range.startLine || range.start?.line || 0) - 1;
  const startCol = (range.startColumn || range.start?.column || 0) - 1;
  const endLine = (range.endLine || range.end?.line || startLine) - 1;
  const endCol = (range.endColumn || range.end?.column || startCol) - 1;

  const maxLine = document.lineCount - 1;
  return new Range(
    new Position(Math.max(0, Math.min(startLine, maxLine)), Math.max(0, startCol)),
    new Position(Math.max(0, Math.min(endLine, maxLine)), Math.max(0, endCol))
  );
}

function mapSeverity(severity: string | number): DiagnosticSeverity {
  if (typeof severity === 'number') {
    switch (severity) {
      case 0: return DiagnosticSeverity.Error;
      case 1: return DiagnosticSeverity.Warning;
      case 2: return DiagnosticSeverity.Information;
      case 3: return DiagnosticSeverity.Hint;
      default: return DiagnosticSeverity.Error;
    }
  }

  switch (String(severity).toLowerCase()) {
    case 'error':
    case 'fatal':
    case 'panic':
      return DiagnosticSeverity.Error;
    case 'warning':
    case 'warn':
      return DiagnosticSeverity.Warning;
    case 'info':
    case 'information':
    case 'note':
      return DiagnosticSeverity.Information;
    case 'hint':
    case 'help':
      return DiagnosticSeverity.Hint;
    default:
      return DiagnosticSeverity.Error;
  }
}

function parseTextOutput(document: TextDocument, output: string): Diagnostic[] {
  const diagnostics: Diagnostic[] = [];
  const lines = output.split('\n');

  const patterns = [
    /^(?:error|ERROR)\[(\w+)\]:\s*(.+?)(?:\s*\[(.+?):(\d+):(\d+)(?:-(\d+)(?::(\d+))?)?\])?$/,
    /^(?:error|ERROR):\s*(.+?)(?:\s+at\s+(.+?):(\d+):(\d+))?$/,
    /^(?:warning|WARNING)\[(\w+)\]:\s*(.+?)(?:\s*\[(.+?):(\d+):(\d+)(?:-(\d+)(?::(\d+))?)?\])?$/,
    /^error\s*-->?\s*(.+?):(\d+):(\d+):\s*(.+)$/,
    /^warning\s*-->?\s*(.+?):(\d+):(\d+):\s*(.+)$/,
    /^(.+?):(\d+):(\d+):\s*(error|warning)\s*:\s*(.+)$/
  ];

  for (const line of lines) {
    if (!line.trim()) continue;

    let matched = false;
    for (const pattern of patterns) {
      const match = line.match(pattern);
      if (match) {
        const diagnostic = parseTextDiagnostic(document, match);
        if (diagnostic) {
          diagnostics.push(diagnostic);
          matched = true;
        }
        break;
      }
    }

    if (!matched && line.includes('error')) {
      const range = new Range(0, 0, 0, 0);
      const diagnostic = new Diagnostic(range, line.trim(), DiagnosticSeverity.Error);
      diagnostic.source = 'fuc';
      diagnostics.push(diagnostic);
    }
  }

  return diagnostics;
}

function parseTextDiagnostic(document: TextDocument, match: RegExpMatchArray): Diagnostic | null {
  let line = 0;
  let col = 0;
  let endLine = 0;
  let endCol = 0;
  let message = '';
  let severity: DiagnosticSeverity = DiagnosticSeverity.Error;

  if (match.length >= 7) {
    const lineNum = parseInt(match[3], 10) - 1;
    const colNum = parseInt(match[4], 10) - 1;
    message = match[2] || match[5] || match[6] || 'Unknown error';
    line = isNaN(lineNum) ? 0 : Math.max(0, lineNum);
    col = isNaN(colNum) ? 0 : Math.max(0, colNum);
    endLine = line;
    endCol = col + 1;

    if (match[1] && match[1].toLowerCase().includes('warning')) {
      severity = DiagnosticSeverity.Warning;
    }
  } else if (match.length >= 5) {
    const lineNum = parseInt(match[2], 10) - 1;
    const colNum = parseInt(match[3], 10) - 1;
    message = match[4] || match[1] || 'Unknown error';
    line = isNaN(lineNum) ? 0 : Math.max(0, lineNum);
    col = isNaN(colNum) ? 0 : Math.max(0, colNum);
    endLine = line;
    endCol = col + 1;

    if (match[1] && !match[1].includes('error')) {
      severity = DiagnosticSeverity.Warning;
    }
  }

  const maxLine = document.lineCount - 1;
  line = Math.min(line, maxLine);
  endLine = Math.min(endLine, maxLine);

  const range = new Range(
    new Position(line, col),
    new Position(endLine, endCol)
  );

  const diagnostic = new Diagnostic(range, message, severity);
  diagnostic.source = 'fuc';
  return diagnostic;
}

function parseCompileErrors(document: TextDocument, error: string): Diagnostic[] {
  const diagnostics: Diagnostic[] = [];
  const lines = error.split('\n');

  for (const line of lines) {
    if (!line.trim()) continue;

    const errorMatch = line.match(/error(?:\[(\w+)\])?:\s*(.+)/i);
    const warningMatch = line.match(/warning(?:\[(\w+)\])?:\s*(.+)/i);

    if (errorMatch) {
      const message = errorMatch[2];
      const code = errorMatch[1];
      const range = extractRangeFromMessage(message, document);
      const diagnostic = new Diagnostic(range, message, DiagnosticSeverity.Error);
      diagnostic.source = 'fuc';
      if (code) {
        diagnostic.code = code;
      }
      diagnostics.push(diagnostic);
    } else if (warningMatch) {
      const message = warningMatch[2];
      const code = warningMatch[1];
      const range = extractRangeFromMessage(message, document);
      const diagnostic = new Diagnostic(range, message, DiagnosticSeverity.Warning);
      diagnostic.source = 'fuc';
      if (code) {
        diagnostic.code = code;
      }
      diagnostics.push(diagnostic);
    } else if (line.toLowerCase().includes('error')) {
      const range = new Range(0, 0, 0, 0);
      const diagnostic = new Diagnostic(range, line.trim(), DiagnosticSeverity.Error);
      diagnostic.source = 'fuc';
      diagnostics.push(diagnostic);
    }
  }

  return diagnostics;
}

function extractRangeFromMessage(message: string, document: TextDocument): Range {
  const patterns = [
    /at\s+(?:line\s+)?(\d+)(?::(\d+))?/,
    /(\d+):(\d+)/,
    /\((\d+),\s*(\d+)\)/
  ];

  for (const pattern of patterns) {
    const match = message.match(pattern);
    if (match) {
      const line = Math.max(0, parseInt(match[1], 10) - 1);
      const col = Math.max(0, (match[2] ? parseInt(match[2], 10) - 1 : 0));
      const maxLine = document.lineCount - 1;
      return new Range(
        new Position(Math.min(line, maxLine), col),
        new Position(Math.min(line, maxLine), col + 1)
      );
    }
  }

  return new Range(0, 0, 0, 0);
}
