import assert from 'node:assert/strict';
import { describe, test } from 'node:test';
import { rewriteArchivedDocContent, rewritePreviewDocContent } from '../scripts/prepare-docs.mjs';

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
});
