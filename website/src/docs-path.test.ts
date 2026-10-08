import assert from 'node:assert/strict';
import { describe, test } from 'node:test';
import { docsRoute, docsPath } from './docs-path.ts';

describe('docsChannel', () => {
  const cases: Array<[string, string]> = [
    ['/docs/', 'stable'],
    ['/docs/preview/', 'preview'],
  ];

  for (const [pathname, expected] of cases) {
    test(`maps ${pathname} to ${expected}`, () => {
      assert.equal(docsRoute(pathname).target, expected);
    });
  }
});

describe('docsPath', () => {
  const cases: Array<[string, string]> = [
    ['index.mdx', 'docs'],
    ['install.mdx', 'docs/install'],
    ['preview/index.mdx', 'docs/preview'],
    ['preview/install.mdx', 'docs/preview/install'],
  ];

  for (const [entry, expected] of cases) {
    test(`maps ${entry} to ${expected}`, () => {
      assert.equal(docsPath({ entry }), expected);
    });
  }
});
