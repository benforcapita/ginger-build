import { test } from 'node:test';
import assert from 'node:assert/strict';
import { languageFor, fileUri, relativeUri } from '../src/language-support.ts';
test('maps supported extensions to LSP IDs and shared server', () => {
  assert.deepEqual(languageFor('src/app.tsx'), { server: 'typescript', language: 'typescriptreact' });
  assert.deepEqual(languageFor('a.py'), { server: 'python', language: 'python' });
  assert.equal(languageFor('a.rs').server, 'rust');
  assert.equal(languageFor('a.jsonc').language, 'jsonc');
  assert.equal(languageFor('notes.txt'), null);
});
test('file URIs roundtrip escaped filenames and reject paths outside workspace', () => {
  const uri = fileUri('/tmp/my project', 'a#?.ts');
  assert.equal(relativeUri('/tmp/my project', uri), 'a#?.ts');
  assert.equal(relativeUri('/tmp/my project', 'file:///tmp/my%20project-evil/file.ts'), null);
  assert.equal(relativeUri('/tmp/my project', 'https://host/file.ts'), null);
  assert.equal(relativeUri('/tmp/my project', 'file://remote/tmp/my%20project/file.ts'), null);
});
test('initialization advertises push diagnostics, not unsupported pull diagnostics', async () => {
  const { prepareLspMessage } = await import('../src/language-support.ts');
  const result = JSON.parse(prepareLspMessage(JSON.stringify({ method: 'initialize', params: { capabilities: { textDocument: { diagnostic: {}, publishDiagnostics: { versionSupport: true } } } } })));
  assert.equal(result.params.capabilities.textDocument.diagnostic, undefined);
  assert.deepEqual(result.params.capabilities.textDocument.publishDiagnostics, { versionSupport: true });
  assert.equal(result.params.capabilities.workspace.configuration, true);
});
