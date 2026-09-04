// Build catalog entries from real Hugging Face metadata. Never invents values:
// every repo, filename and byte size below is read from the live HF API, and a
// repo that cannot be resolved aborts without replacing the last good catalog.
//   node scripts/build_catalog.mjs          # preview JSON on stdout
//   node scripts/build_catalog.mjs --write  # atomically replace catalog/catalog.json
import { rename, writeFile } from "node:fs/promises";
import { createPrivateKey, sign } from "node:crypto";

const WANTED = [
  { repo: 'unsloth/Qwen3.8-27B-GGUF', prefer: ['Q4_K_M', 'Q5_K_M', 'Q8_0'], family: 'Qwen3.8', params: '27B', tags: ['general', 'reasoning'] },
  { repo: 'unsloth/Qwen3-Coder-30B-A3B-Instruct-GGUF', prefer: ['Q4_K_M', 'Q6_K'], family: 'Qwen3 Coder', params: '30B-A3B', tags: ['code', 'moe'] },
  { repo: 'bartowski/Llama-3.2-1B-Instruct-GGUF', prefer: ['Q4_K_M', 'Q8_0'], family: 'Llama 3.2', params: '1B', tags: ['small', 'general'] },
  { repo: 'unsloth/Llama-3.2-3B-Instruct-GGUF', prefer: ['Q4_K_M', 'Q8_0'], family: 'Llama 3.2', params: '3B', tags: ['small', 'general'] },
  { repo: 'unsloth/gemma-4-31B-it-GGUF', prefer: ['Q4_K_M', 'Q6_K'], family: 'Gemma 4', params: '31B', tags: ['general'] },
  { repo: 'unsloth/Phi-4-mini-instruct-GGUF', prefer: ['Q4_K_M', 'Q8_0'], family: 'Phi-4', params: 'mini', tags: ['small', 'general'] },
  { repo: 'MaziyarPanahi/phi-4-GGUF', prefer: ['Q4_K_M', 'Q6_K'], family: 'Phi-4', params: '14B', tags: ['general'] },
  { repo: 'lmstudio-community/Qwen3.8-27B-GGUF', prefer: ['Q4_K_M'], family: 'Qwen3.8', params: '27B', tags: ['general'] },
];

const api = async (url) => {
  const r = await fetch(url, { headers: { 'User-Agent': 'localmotive-catalog-builder' } });
  if (!r.ok) throw new Error(`${r.status} ${url}`);
  return r.json();
};

const entries = [];
const problems = [];

for (const want of WANTED) {
  try {
    const meta = await api(`https://huggingface.co/api/models/${want.repo}?blobs=true`);
    const ggufs = (meta.siblings || []).filter((s) => s.rfilename.toLowerCase().endsWith('.gguf'));
    if (!ggufs.length) { problems.push(`${want.repo}: no .gguf files`); continue; }

    const files = [];
    for (const quant of want.prefer) {
      // Single-file build for this quant, ignoring split shards and companions.
      const match = ggufs.find((s) => {
        const n = s.rfilename;
        return n.toUpperCase().includes(quant.toUpperCase())
          && !/-\d{5}-of-\d{5}\./.test(n)
          && !/mmproj|dspark|dflash|eagle3|-mtp-/i.test(n);
      });
      if (!match) continue;
      if (typeof match.size !== 'number' || match.size <= 0) { problems.push(`${want.repo}/${match.rfilename}: no size`); continue; }
      const sha256 = match.lfs?.sha256 ?? match.lfs?.oid;
      if (!/^[a-f0-9]{64}$/i.test(sha256 ?? '')) { problems.push(`${want.repo}/${match.rfilename}: no SHA-256`); continue; }
      files.push({ quant, filename: match.rfilename, sizeBytes: match.size, sha256 });
    }
    if (!files.length) { problems.push(`${want.repo}: none of ${want.prefer.join('/')} found`); continue; }

    entries.push({
      id: want.repo.toLowerCase().replace(/[^a-z0-9]+/g, '-'),
      repo: want.repo,
      family: want.family,
      parameters: want.params,
      publisher: want.repo.split('/')[0],
      summary: (meta.cardData && meta.cardData.summary) || '',
      tags: want.tags,
      gated: Boolean(meta.gated),
      downloads: meta.downloads ?? 0,
      likes: meta.likes ?? 0,
      files,
    });
  } catch (error) {
    problems.push(`${want.repo}: ${error.message}`);
  }
}

const catalog = {
  schemaVersion: 1,
  updated: new Date().toISOString().slice(0, 10),
  source: 'https://github.com/sato942/localmotive/blob/main/catalog/catalog.json',
  note: 'Curated list of GGUF builds. Localmotive downloads directly from Hugging Face; this file only decides what is offered.',
  models: entries.sort((a, b) => b.downloads - a.downloads),
};
const rendered = JSON.stringify(catalog, null, 2) + '\n';
console.error(`resolved ${entries.length}/${WANTED.length} repos, ${entries.reduce((n, e) => n + e.files.length, 0)} files`);

if (problems.length) {
  console.error('UNRESOLVED (catalog not replaced):\n  ' + problems.join('\n  '));
  process.exitCode = 1;
} else if (process.argv.includes('--write')) {
  const signingKey = process.env.LOCALMOTIVE_CATALOG_SIGNING_KEY_PEM;
  if (!signingKey) {
    console.error('LOCALMOTIVE_CATALOG_SIGNING_KEY_PEM is required to publish the catalog');
    process.exitCode = 1;
    process.exit();
  }
  const signature = sign(null, Buffer.from(rendered), createPrivateKey(signingKey)).toString('base64');
  const temporary = 'catalog/catalog.json.next';
  const signatureTemporary = 'catalog/catalog.json.sig.next';
  await writeFile(temporary, rendered, 'utf8');
  await writeFile(signatureTemporary, signature, 'utf8');
  await rename(temporary, 'catalog/catalog.json');
  await rename(signatureTemporary, 'catalog/catalog.json.sig');
  console.error('wrote catalog/catalog.json and its Ed25519 signature');
} else {
  process.stdout.write(rendered);
}
