import assert from 'node:assert/strict';
import { describe, test } from 'node:test';
import { rewriteArchivedDocContent, rewriteArchivedDocLinks, rewritePreviewDocContent } from '../scripts/prepare-docs.mjs';

describe('rewritePreviewDocContent', () => {
  test('adds one parent segment to frontmatter file: public assets', () => {
    const input = ['hero:', '  image:', '    file: ../../../public/assets/logo.svg'].join('\n');
    const output = rewritePreviewDocContent(input, 'index.mdx');
    assert.match(output, /file: \.\.\/\.\.\/\.\.\/\.\.\/public\/assets\/logo\.svg/);
  });

  test('rewrites deeper locale frontmatter asset paths too', () => {
    const input = '    file: ../../../../public/assets/logo.svg';
    const output = rewritePreviewDocContent(input, 'ja/index.mdx');
    assert.match(output, /file: (?:\.\.\/){5}public\/assets\/logo\.svg/);
  });

  test('leaves unrelated URLs containing ../public/ unchanged', () => {
    const input = 'See https://example.com/../public/logo.svg for details.';
    const output = rewritePreviewDocContent(input, 'index.mdx');
    assert.ok(
      output.includes('https://example.com/../public/logo.svg'),
      `unrelated URL was rewritten: ${output}`,
    );
  });

  test('leaves ../public/ mentioned in prose unchanged', () => {
    const input = 'The asset lives under ../public/assets in the repo.';
    const output = rewritePreviewDocContent(input, 'index.mdx');
    assert.ok(
      output.includes('under ../public/assets'),
      `prose path was rewritten: ${output}`,
    );
  });
});


describe('rewriteArchivedDocContent', () => {
  test('rewrites root archive assets once and is idempotent', () => {
    const input = 'file: ../../../public/assets/logo.svg\n';
    const output = rewriteArchivedDocContent(input, false);
    assert.equal(output, 'file: ../../../../../public/assets/logo.svg\n');
    assert.equal(rewriteArchivedDocContent(output, false), output);
  });

  test('rewrites localized archive assets once and is idempotent', () => {
    const input = 'file: ../../../../public/assets/logo.svg\n';
    const output = rewriteArchivedDocContent(input, true);
    assert.equal(output, 'file: ../../../../../../public/assets/logo.svg\n');
    assert.equal(rewriteArchivedDocContent(output, true), output);
  });

  test('rewrites root archive component imports once and is idempotent', () => {
    const input = "import MobileDocShots from '../../components/MobileDocShots.astro';";
    const output = rewriteArchivedDocContent(input, false);
    assert.equal(output, "import MobileDocShots from '../../../../components/MobileDocShots.astro';");
    assert.equal(rewriteArchivedDocContent(output, false), output);
  });

  test('rewrites localized archive component imports once and is idempotent', () => {
    const input = "import MobileDocShots from '../../../components/MobileDocShots.astro';";
    const output = rewriteArchivedDocContent(input, true);
    assert.equal(output, "import MobileDocShots from '../../../../../components/MobileDocShots.astro';");
    assert.equal(rewriteArchivedDocContent(output, true), output);
  });

  test('rewrites stale upstream references once and is idempotent', () => {
    const input = [
      'nix run github:ogulcancelik/herdr/v0.8.0',
      'herdr plugin install ogulcancelik/herdr-plugin-examples/agent-telegram-notify',
      '{"owner":"ogulcancelik","repo":"herdr-plugin-examples"}',
      'the docs live at herdr.dev',
    ].join('\n');
    const expected = [
      'nix run github:GroepOnline/herdr/v0.8.0',
      'herdr plugin install GroepOnline/herdr-plugin-examples/agent-telegram-notify',
      '{"owner":"GroepOnline","repo":"herdr-plugin-examples"}',
      'the docs live at herdr.chefgroep.nl',
    ].join('\n');
    const output = rewriteArchivedDocContent(input, false);
    assert.equal(output, expected);
    assert.equal(rewriteArchivedDocContent(output, false), output);
  });
});

describe('rewriteArchivedDocLinks', () => {
  test('prefixes root archive links with the version and is idempotent', () => {
    const input = 'See [fleet ops](/docs/fleet-ops/) and [moshi](/docs/moshi/).';
    const pages = { root: ['fleet-ops', 'moshi'], ja: ['fleet-ops'], 'zh-cn': ['fleet-ops'] };
    const output = rewriteArchivedDocLinks(input, '0.7.7', pages);
    assert.equal(output, 'See [fleet ops](/docs/0.7.7/fleet-ops/) and [moshi](/docs/0.7.7/moshi/).');
    assert.equal(rewriteArchivedDocLinks(output, '0.7.7', pages), output);
  });

  test('prefixes localized archive links with the version', () => {
    const input = '[kansetsu](/ja/docs/moshi/)';
    const pages = { root: [], ja: ['moshi'], 'zh-cn': [] };
    const output = rewriteArchivedDocLinks(input, '0.7.7', pages);
    assert.equal(output, '[kansetsu](/ja/docs/0.7.7/moshi/)');
    assert.equal(rewriteArchivedDocLinks(output, '0.7.7', pages), output);
  });

  test('leaves links to pages not in the archive unchanged', () => {
    const input = '[install](/docs/install/)';
    const output = rewriteArchivedDocLinks(input, '0.7.7', { root: ['fleet-ops'] });
    assert.equal(output, input);
  });
});
