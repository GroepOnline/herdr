import { cp, mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import process from 'node:process';

const websiteDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(websiteDir, '../..');
const publicDir = resolve(repoRoot, 'website/public');
const stableDocsDir = resolve(repoRoot, 'website/src/content/docs');
const previewDocsSourceDir = resolve(repoRoot, 'docs/next/website/src/content/docs');
const previewDocsDir = resolve(stableDocsDir, 'preview');
const previewConfigReferenceSource = resolve(
  repoRoot,
  'docs/next/website/src/data/config-reference.json',
);
const previewConfigReferenceDestination = resolve(
  repoRoot,
  'website/src/data/config-reference-preview.json',
);

if (process.argv[2] === '--rewrite-preview-doc-fixture') {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  process.stdout.write(rewritePreviewDocContent(Buffer.concat(chunks).toString('utf8')));
} else {
  await preparePublicAssets();
  await preparePreviewDocs();
  await prepareDocsVersions();
}

async function prepareDocsVersions() {
  const manifest = JSON.parse(await readFile(resolve(repoRoot, 'docs/versions/manifest.json'), 'utf8'));
  const versions = [];
  const scopes = {};
  const references = {};
  for (const entry of manifest.versions) {
    const source = resolve(repoRoot, 'docs/versions', entry.version, 'website/src/content/docs');
    const destination = resolve(stableDocsDir, '_versions', entry.version);
    const locales = {};
    await collectDocPages(source, '', locales);
    await rm(destination, { recursive: true, force: true });
    await cp(source, destination, { recursive: true });
    await rewriteArchivedDocs(destination, entry.version, locales);
    versions.push({ version: entry.version, tag: entry.tag });
    scopes[entry.version] = { locales };
    try {
      references[entry.version] = JSON.parse(await readFile(resolve(repoRoot, 'docs/versions', entry.version, 'website/src/data/config-reference.json'), 'utf8'));
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
    }
  }
  await writeFile(resolve(repoRoot, 'website/src/data/docs-versions.json'), `${JSON.stringify({ current: manifest.current, versions, scopes }, null, 2)}\n`, 'utf8');
  await writeFile(resolve(repoRoot, 'website/src/data/config-reference-versions.json'), `${JSON.stringify(references, null, 2)}\n`, 'utf8');
}

export function rewriteArchivedDocContent(content, isLocalized) {
  return content
    .replaceAll('herdr.dev', 'herdr.chefgroep.nl')
    .replaceAll('github.com/ogulcancelik/herdr', 'github.com/GroepOnline/herdr')
    .replace(
      /^(\s*file:\s*["']?)((?:\.\.\/){3,4}public\/assets\/)/gm,
      (_match, prefix, assetPath) => {
        const expectedSegments = isLocalized ? 4 : 3;
        const actualSegments = (assetPath.match(/\.\.\//g) ?? []).length;
        return actualSegments === expectedSegments ? `${prefix}../../${assetPath}` : _match;
      },
    )
    .replace(
      /^(import .*from\s+['"])(?=(?:\.\.\/){2,3}components\/)/gm,
      (_match, prefix) => `${prefix}../../`,
    );
}

async function rewriteArchivedDocs(directory, version, locales) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await rewriteArchivedDocs(path, version, locales);
    else if (entry.isFile()) {
      const content = await readFile(path, 'utf8');
      const isLocalized = /[\\/](?:ja|zh-cn)[\\/]/.test(path);
      const rewritten = rewriteArchivedDocContent(content, isLocalized);
      await writeFile(path, rewriteArchivedDocLinks(rewritten, version, locales), 'utf8');
    }
  }
}

export function rewriteArchivedDocLinks(content, version, archivePages = {}) {
  let rewritten = content;
  for (const page of archivePages.root ?? []) {
    rewritten = rewritten.split(`/docs/${page}/`).join(`/docs/${version}/${page}/`);
  }
  for (const locale of ['ja', 'zh-cn']) {
    for (const page of archivePages[locale] ?? []) {
      rewritten = rewritten
        .split(`/${locale}/docs/${page}/`)
        .join(`/${locale}/docs/${version}/${page}/`);
    }
  }
  return rewritten;
}

async function collectDocPages(directory, relativeDirectory, locales) {
  for (const entry of await readdir(resolve(directory, relativeDirectory), { withFileTypes: true })) {
    const relativePath = join(relativeDirectory, entry.name);
    if (entry.isDirectory()) await collectDocPages(directory, relativePath, locales);
    else if (/\.(md|mdx|markdown|mdown|mkdn|mkd|mdwn)$/i.test(entry.name)) {
      const parts = relativePath.split('/');
      const locale = ['ja', 'zh-cn'].includes(parts[0]) ? parts.shift() : 'root';
      const page = parts.join('/').replace(/\.(md|mdx|markdown|mdown|mkdn|mkd|mdwn)$/i, '').replace(/\/index$/, '') || 'index';
      (locales[locale] ??= []).push(page);
    }
  }
}

async function preparePublicAssets() {
  await rm(publicDir, { recursive: true, force: true });
  await mkdir(publicDir, { recursive: true });

  for (const file of [
    'install.sh',
    'install.ps1',
    'agent-guide.md',
    'latest.json',
    'preview.json',
    'dev.json',
    'robots.txt',
    '_headers',
    '_redirects',
  ]) {
    const source = resolve(repoRoot, 'website', file);
    const optional = file === 'preview.json' || file === 'dev.json';
    try {
      await cp(source, resolve(publicDir, file));
    } catch (error) {
      if (!optional || error.code !== 'ENOENT') throw error;
    }
  }

  for (const directory of ['assets', 'css', 'agent-detection']) {
    await cp(resolve(repoRoot, 'website', directory), resolve(publicDir, directory), {
      recursive: true,
    });
  }
}

async function preparePreviewDocs() {
  await rm(previewDocsDir, { recursive: true, force: true });
  await copyPreviewDocs(previewDocsSourceDir, previewDocsDir);
  await cp(previewConfigReferenceSource, previewConfigReferenceDestination);
}

async function copyPreviewDocs(sourceDir, destinationDir) {
  await mkdir(destinationDir, { recursive: true });
  for (const entry of await readdir(sourceDir, { withFileTypes: true })) {
    const source = join(sourceDir, entry.name);
    const destination = join(destinationDir, entry.name);
    if (entry.isDirectory()) {
      await copyPreviewDocs(source, destination);
      continue;
    }
    if (!entry.isFile()) continue;

    const content = await readFile(source, 'utf8');
    const relativePath = relative(previewDocsSourceDir, source);
    await writeFile(destination, rewritePreviewDocContent(content, relativePath), 'utf8');
  }
}

export function rewritePreviewDocContent(content, relativePath = '') {
  const rewritten = content
    .replaceAll('/docs/', '/docs/preview/')
    // Splash-hero frontmatter references bundled assets with a relative `file:`
    // path into `public/`. Preview docs live one directory deeper, so only
    // these asset paths gain an extra `../`. Anchoring to the `file:` key (at
    // line start, optionally quoted) avoids rewriting `../public/` substrings
    // inside unrelated URLs or prose such as https://example.com/../public/x.
    .replace(
      /^(\s*file:\s*["']?)((?:\.\.\/)+public\/)/gm,
      (_match, prefix, assetPath) => `${prefix}../${assetPath}`,
    )
    // Preview docs live one directory deeper than stable docs, so component
    // imports need one more parent segment regardless of locale depth. Only
    // MDX import lines are rewritten; prose mentioning relative paths is not.
    .replace(/^(import .*from\s+['"])(?=(?:\.\.\/)+components\/)/gm, '$1../');
  return insertPreviewNotice(rewritten, relativePath);
}

function insertPreviewNotice(content, relativePath) {
  const notice = [
    '> Preview docs describe unreleased preview builds. Stable docs remain at [/docs/](/docs/).',
    '',
    '',
  ].join('\n');
  const indexPrefix =
    relativePath === 'index.mdx'
      ? content.replace('title: Herdr documentation', 'title: Herdr preview documentation')
      : content;
  const frontmatter = indexPrefix.match(/^---\n[\s\S]*?\n---\n/);
  if (!frontmatter) {
    return insertNoticeAfterImports(indexPrefix, notice);
  }
  const body = indexPrefix.slice(frontmatter[0].length);
  return `${frontmatter[0]}\n${insertNoticeAfterImports(body, notice)}`;
}

function insertNoticeAfterImports(content, notice) {
  const imports = content.match(/^(\s*import .+?;\n)+\s*/);
  if (!imports) {
    return `${notice}${content}`;
  }
  return `${imports[0]}${notice}${content.slice(imports[0].length)}`;
}
