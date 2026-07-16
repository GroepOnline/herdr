import assert from 'node:assert/strict';
import { describe, test } from 'node:test';
import { docsChannel, docsPath } from './docs-path.ts';

describe('docsChannel', () => {
  const cases: Array<[string, string]> = [
    ['/docs/', 'stable'],
    ['/ja/docs/install/', 'stable'],
    ['/zh-cn/docs/', 'stable'],
    ['/docs/preview/', 'preview'],
    ['/ja/docs/preview/install/', 'preview'],
    ['/zh-cn/docs/preview/', 'preview'],
  ];

  for (const [pathname, expected] of cases) {
    test(`maps ${pathname} to ${expected}`, () => {
      assert.equal(docsChannel(pathname), expected);
    });
  }
});

describe('docsPath', () => {
  const cases: Array<[string, string]> = [
    ['index.mdx', 'docs'],
    ['install.mdx', 'docs/install'],
    ['ja/index.mdx', 'ja/docs'],
    ['ja/install.mdx', 'ja/docs/install'],
    ['zh-cn/install.mdx', 'zh-cn/docs/install'],
    ['preview/index.mdx', 'docs/preview'],
    ['preview/install.mdx', 'docs/preview/install'],
    ['preview/ja/index.mdx', 'ja/docs/preview'],
    ['preview/ja/install.mdx', 'ja/docs/preview/install'],
    ['preview/zh-cn/install.mdx', 'zh-cn/docs/preview/install'],
  ];

  for (const [entry, expected] of cases) {
    test(`maps ${entry} to ${expected}`, () => {
      assert.equal(docsPath({ entry }), expected);
    });
  }
});
