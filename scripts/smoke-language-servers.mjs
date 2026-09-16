// Optional integration smoke: requires the tools shown in Ginger's Language servers panel.
import { spawn, execFileSync } from 'node:child_process';
import { mkdtemp, writeFile, mkdir, rm } from 'node:fs/promises';
import { tmpdir, homedir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';
const root = await mkdtemp(join(tmpdir(), 'ginger-lsp-smoke-'));
const managed = join(homedir(), '.ginger/language-servers/node_modules/.bin');
const configs = [
  { id: 'typescript', executable: join(managed, 'typescript-language-server'), args: ['--stdio'], languageId: 'typescript', file: 'sample.ts', text: 'const greeting: string = 123;\ngreeting.', position: { line: 1, character: 9 }, label: 'charAt' },
  { id: 'python', executable: join(managed, 'pyright-langserver'), args: ['--stdio'], languageId: 'python', file: 'sample.py', text: 'value: str = 1\nvalue.', position: { line: 1, character: 6 }, label: 'capitalize' },
  { id: 'json', executable: join(managed, 'vscode-json-language-server'), args: ['--stdio'], languageId: 'json', file: 'sample.json', text: '{"he": }', position: { line: 0, character: 4 }, label: 'hello' },
  { id: 'rust', executable: execFileSync('rustup', ['which', 'rust-analyzer'], { encoding: 'utf8' }).trim(), args: [], languageId: 'rust', file: 'src/main.rs', text: 'fn main() { let value = String::new(); value. }', position: { line: 0, character: 45 }, label: 'len' },
];
try {
  await mkdir(join(root, 'src'));
  await writeFile(join(root, 'Cargo.toml'), '[package]\nname="ginger_lsp_smoke"\nversion="0.1.0"\nedition="2021"\n');
  await writeFile(join(root, 'tsconfig.json'), '{"compilerOptions":{"strict":true}}');
  for (const cfg of configs) {
    await writeFile(join(root, cfg.file), cfg.text);
    const child = spawn(cfg.executable, cfg.args, { detached: true, cwd: root, env: { ...process.env, PATH: `${execFileSync('rustup', ['which', 'cargo'], { encoding: 'utf8' }).trim().replace(/\/cargo$/, '')}:${process.env.PATH}` } });
    let buffer = Buffer.alloc(0), next = 0, stderr = '', sawDiagnostics = false;
    const pending = new Map();
    const send = value => { const text = JSON.stringify(value); child.stdin.write(`Content-Length: ${Buffer.byteLength(text)}\r\n\r\n${text}`); };
    const notification = (method, params) => send({ jsonrpc: '2.0', method, params });
    const request = (method, params) => new Promise((resolve, reject) => {
      const id = ++next;
      const timer = setTimeout(() => { pending.delete(id); reject(new Error(`${cfg.id}: timeout ${method}: ${stderr}`)); }, 30000);
      pending.set(id, { resolve: result => { clearTimeout(timer); resolve(result); }, reject: error => { clearTimeout(timer); reject(error); } });
      send({ jsonrpc: '2.0', id, method, params });
    });
    child.stderr.on('data', chunk => { stderr = (stderr + chunk).slice(-4000); });
    child.on('error', error => { for (const req of pending.values()) req.reject(error); pending.clear(); });
    child.stdout.on('data', chunk => {
      buffer = Buffer.concat([buffer, chunk]);
      while (true) {
        const boundary = buffer.indexOf('\r\n\r\n'); if (boundary < 0) break;
        const length = Number(/Content-Length:\s*(\d+)/i.exec(buffer.subarray(0, boundary).toString())?.[1]);
        if (buffer.length < boundary + 4 + length) break;
        const value = JSON.parse(buffer.subarray(boundary + 4, boundary + 4 + length).toString()); buffer = buffer.subarray(boundary + 4 + length);
        if (value.method && value.id !== undefined) send({ jsonrpc: '2.0', id: value.id, result: value.method === 'workspace/configuration' ? value.params.items.map(() => ({})) : null });
        else if (value.id !== undefined) { const req = pending.get(value.id); pending.delete(value.id); if (value.error) req?.reject(new Error(JSON.stringify(value.error))); else req?.resolve(value.result); }
        else if (value.method === 'textDocument/publishDiagnostics' && value.params.diagnostics.length) sawDiagnostics = true;
      }
    });
    try {
      const result = await request('initialize', { processId: process.pid, rootUri: pathToFileURL(root).href, initializationOptions: cfg.id === 'typescript' ? { tsserver: { fallbackPath: join(managed, '../typescript/lib/tsserver.js') } } : {}, capabilities: { textDocument: { completion: { completionItem: { snippetSupport: true } }, publishDiagnostics: {} }, workspace: { configuration: true } } });
      assert.ok(result.capabilities.completionProvider, `${cfg.id}: completion capability`);
      notification('initialized', {});
      notification('workspace/didChangeConfiguration', { settings: { python: { analysis: { typeCheckingMode: 'basic' } }, json: { validate: { enable: true }, schemas: [{ fileMatch: ['*.json'], schema: { type: 'object', properties: { hello: { type: 'string' } } } }] } } });
      const uri = pathToFileURL(join(root, cfg.file)).href;
      notification('textDocument/didOpen', { textDocument: { uri, languageId: cfg.languageId, version: 1, text: cfg.text } });
      let items = [];
      for (let attempt = 0; attempt < 15; attempt++) {
        const completion = await request('textDocument/completion', { textDocument: { uri }, position: cfg.position });
        items = Array.isArray(completion) ? completion : completion?.items ?? [];
        if (items.some(item => item.label.includes(cfg.label))) break;
        await new Promise(resolve => setTimeout(resolve, 500));
      }
      assert.ok(items.some(item => item.label.includes(cfg.label)), `${cfg.id}: expected ${cfg.label}, got ${items.slice(0, 10).map(i => i.label)}`);
      {
        for (let attempt = 0; !sawDiagnostics && attempt < 20; attempt++) await new Promise(resolve => setTimeout(resolve, 250));
        assert.ok(sawDiagnostics, `${cfg.id}: diagnostics`);
      }
      console.log(`${cfg.id}: initialized, completion ${cfg.label}${sawDiagnostics ? ', diagnostics' : ''}`);
      notification('textDocument/didClose', { textDocument: { uri } });
      await request('shutdown', null); notification('exit');
    } finally { if (child.pid) { try { process.kill(-child.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; } } for (const req of pending.values()) req.reject(new Error('Test ended')); pending.clear(); }
  }
} finally { await rm(root, { recursive: true, force: true }); }
