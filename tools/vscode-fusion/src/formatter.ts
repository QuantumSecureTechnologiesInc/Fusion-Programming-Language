import {
  DocumentFormattingEditProvider,
  DocumentRangeFormattingEditProvider,
  FormattingOptions,
  TextDocument,
  CancellationToken,
  ProviderResult,
  TextEdit,
  workspace,
  Range,
  Position
} from 'vscode';
import { execFile } from 'child_process';
import { promisify } from 'util';

const execFileAsync = promisify(execFile);

export function createFormatter(): DocumentFormattingEditProvider & DocumentRangeFormattingEditProvider {
  return {
    provideDocumentFormattingEdits(
      document: TextDocument,
      options: FormattingOptions,
      token: CancellationToken
    ): ProviderResult<TextEdit[]> {
      return formatDocument(document, options, token);
    },

    provideDocumentRangeFormattingEdits(
      document: TextDocument,
      range: Range,
      options: FormattingOptions,
      token: CancellationToken
    ): ProviderResult<TextEdit[]> {
      return formatRange(document, range, options, token);
    }
  };
}

async function formatDocument(
  document: TextDocument,
  options: FormattingOptions,
  token: CancellationToken
): Promise<TextEdit[]> {
  const config = workspace.getConfiguration('fusion');
  const fucPath = config.get<string>('fucPath', 'fuc');

  try {
    const result = await runFucFmt(document, fucPath, options);
    if (result !== null) {
      return [TextEdit.replace(fullDocumentRange(document), result)];
    }
  } catch (error) {
    // Fallback to built-in formatting
  }

  return builtInFormat(document, options);
}

async function formatRange(
  document: TextDocument,
  range: Range,
  options: FormattingOptions,
  token: CancellationToken
): Promise<TextEdit[]> {
  const config = workspace.getConfiguration('fusion');
  const fucPath = config.get<string>('fucPath', 'fuc');

  try {
    const result = await runFucFmt(document, fucPath, options);
    if (result !== null) {
      return [TextEdit.replace(range, result)];
    }
  } catch (error) {
    // Fallback to built-in formatting
  }

  return builtInRangeFormat(document, range, options);
}

async function runFucFmt(
  document: TextDocument,
  fucPath: string,
  options: FormattingOptions
): Promise<string | null> {
  const documentText = document.getText();
  const args = ['fmt', '--stdin', '--indent', options.insertSpaces ? String(options.tabSize) : 'tab'];

  const result = await execFileAsync(fucPath, args, {
    input: documentText,
    timeout: 5000,
    maxBuffer: 1024 * 1024
  });

  if (result.stdout) {
    return result.stdout;
  }

  return null;
}

function fullDocumentRange(document: TextDocument): Range {
  const lastLine = document.lineCount - 1;
  return new Range(
    new Position(0, 0),
    new Position(lastLine, document.lineAt(lastLine).text.length)
  );
}

function builtInFormat(
  document: TextDocument,
  options: FormattingOptions
): TextEdit[] {
  const edits: TextEdit[] = [];
  const tabSize = options.tabSize;
  const insertSpaces = options.insertSpaces;
  const indentStr = insertSpaces ? ' '.repeat(tabSize) : '\t';

  let indentLevel = 0;
  let inString = false;
  let stringChar = '';
  let inLineComment = false;
  let inBlockComment = false;

  for (let i = 0; i < document.lineCount; i++) {
    const line = document.lineAt(i);
    const text = line.text;
    const trimmed = text.trimStart();

    if (trimmed.length === 0) {
      if (text.length > 0) {
        edits.push(TextEdit.replace(line.range, ''));
      }
      continue;
    }

    let adjustedIndent = indentLevel;

    const firstNonSpace = text.search(/\S/);
    const leadingChars = firstNonSpace >= 0 ? text.substring(0, firstNonSpace) : text;

    const decreasesIndent = /^[\}\)\]]/.test(trimmed) ||
      /^else\b/.test(trimmed) ||
      /^elif\b/.test(trimmed) ||
      /^catch\b/.test(trimmed);

    if (decreasesIndent && indentLevel > 0) {
      adjustedIndent = Math.max(0, indentLevel - 1);
    }

    const newIndent = indentStr.repeat(adjustedIndent);
    const trimmedText = trimmed;

    const lineText = trimmedText;
    let opens = 0;
    let closes = 0;

    let inStr = false;
    let strCh = '';
    let inLC = false;
    let inBC = false;

    for (let j = 0; j < lineText.length; j++) {
      const ch = lineText[j];
      const next = j + 1 < lineText.length ? lineText[j + 1] : '';

      if (inLC) break;

      if (inBC) {
        if (ch === '*' && next === '/') {
          inBC = false;
          j++;
        }
        continue;
      }

      if (inStr) {
        if (ch === '\\') {
          j++;
          continue;
        }
        if (ch === strCh) {
          inStr = false;
        }
        continue;
      }

      if (ch === '/' && next === '/') {
        inLC = true;
        continue;
      }
      if (ch === '/' && next === '*') {
        inBC = true;
        j++;
        continue;
      }
      if (ch === '"' || ch == '\'') {
        inStr = true;
        strCh = ch;
        continue;
      }

      if (ch === '{' || ch === '(' || ch === '[') {
        opens++;
      } else if (ch === '}' || ch === ')' || ch === ']') {
        closes++;
      }
    }

    if (newIndent !== leadingChars || trimmedText !== text.trim()) {
      edits.push(TextEdit.replace(line.range, newIndent + trimmedText));
    }

    indentLevel += opens - closes;
    indentLevel = Math.max(0, indentLevel);
  }

  return edits;
}

function builtInRangeFormat(
  document: TextDocument,
  range: Range,
  options: FormattingOptions
): TextEdit[] {
  const edits: TextEdit[] = [];
  const tabSize = options.tabSize;
  const insertSpaces = options.insertSpaces;
  const indentStr = insertSpaces ? ' '.repeat(tabSize) : '\t';

  const startLine = range.start.line;
  const endLine = range.end.line;

  let baseIndent = 0;
  for (let i = 0; i < startLine; i++) {
    const line = document.lineAt(i);
    const text = line.text.trim();
    const opens = (text.match(/[\{\(\[]/g) || []).length;
    const closes = (text.match(/[\}\)\]]/g) || []).length;
    baseIndent += opens - closes;
  }
  baseIndent = Math.max(0, baseIndent);

  let indentLevel = baseIndent;

  for (let i = startLine; i <= endLine; i++) {
    const line = document.lineAt(i);
    const text = line.text;
    const trimmed = text.trimStart();

    if (trimmed.length === 0) {
      continue;
    }

    let adjustedIndent = indentLevel;

    const trimmedText = trimmed;
    const decreasesIndent = /^[\}\)\]]/.test(trimmedText) ||
      /^else\b/.test(trimmedText) ||
      /^elif\b/.test(trimmedText);

    if (decreasesIndent && indentLevel > baseIndent) {
      adjustedIndent = Math.max(baseIndent, indentLevel - 1);
    }

    const newIndent = indentStr.repeat(adjustedIndent);
    const leadingChars = text.substring(0, text.search(/\S/));

    if (newIndent !== leadingChars) {
      edits.push(TextEdit.replace(line.range, newIndent + trimmedText));
    }

    let opens = 0;
    let closes = 0;

    for (const ch of trimmedText) {
      if (ch === '{' || ch === '(' || ch === '[') opens++;
      if (ch === '}' || ch === ')' || ch === ']') closes++;
    }

    indentLevel += opens - closes;
    indentLevel = Math.max(baseIndent, indentLevel);
  }

  return edits;
}
