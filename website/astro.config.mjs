import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { PRODUCT_GITHUB_REPO, PRODUCT_SITE_URL, productGitHubUrl } from './src/config/product.ts';

const repoBlob = `${productGitHubUrl('/blob/main/')}`;

function rewriteHerdrLinks() {
  const docsLinks = new Map([
    ['README.md', '/docs/'],
    ['./README.md', '/docs/'],
    ['CONFIGURATION.md', '/docs/configuration/'],
    ['./CONFIGURATION.md', '/docs/configuration/'],
    ['INTEGRATIONS.md', '/docs/integrations/'],
    ['./INTEGRATIONS.md', '/docs/integrations/'],
    ['SOCKET_API.md', '/docs/socket-api/'],
    ['./SOCKET_API.md', '/docs/socket-api/'],
    ['SKILL.md', '/docs/agent-skill/'],
    ['./SKILL.md', '/docs/agent-skill/'],
  ]);

  return function transform(tree) {
    walk(tree, (node) => {
      if (!node || (node.type !== 'link' && node.type !== 'definition')) return;
      if (typeof node.url !== 'string') return;

      const [path, suffix = ''] = node.url.split(/(?=[#?])/);
      const mapped = docsLinks.get(path);
      if (mapped) {
        node.url = `${mapped}${suffix}`;
        return;
      }

      const sourcePath = path.startsWith('./') ? path.slice(2) : path;
      if (
        sourcePath.startsWith('src/') ||
        sourcePath.startsWith('scripts/') ||
        sourcePath.startsWith('assets/')
      ) {
        node.url = `${repoBlob}${sourcePath}${suffix}`;
      }
    });
  };
}

function walk(node, visitor) {
  visitor(node);
  if (!node || !Array.isArray(node.children)) return;
  for (const child of node.children) walk(child, visitor);
}

export default defineConfig({
  site: PRODUCT_SITE_URL,
  integrations: [
    starlight({
      title: 'herdr',
      description: 'Terminal-native agent runtime and multiplexer.',
      favicon: '/assets/favicon.png?v=14',
      defaultLocale: 'root',
      locales: {
        root: { label: 'English', lang: 'en' },
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: productGitHubUrl(),
        },
      ],
      components: {
        Banner: './src/components/Banner.astro',
        Head: './src/components/Head.astro',
        Header: './src/components/Header.astro',
        Search: './src/components/Search.astro',
        Sidebar: './src/components/Sidebar.astro',
        SiteTitle: './src/components/SiteTitle.astro',
      },
      customCss: ['./src/styles/starlight.css'],
      head: [
        {
          tag: 'meta',
          attrs: { property: 'og:image', content: `${PRODUCT_SITE_URL}/assets/og-card-v8.png` },
        },
        { tag: 'meta', attrs: { property: 'og:image:width', content: '1200' } },
        { tag: 'meta', attrs: { property: 'og:image:height', content: '630' } },
        {
          tag: 'meta',
          attrs: {
            property: 'og:image:alt',
            content: 'Herdr documentation — One terminal. The whole herd.',
          },
        },
        {
          tag: 'meta',
          attrs: { name: 'twitter:image', content: `${PRODUCT_SITE_URL}/assets/og-card-v8.png` },
        },
        {
          tag: 'meta',
          attrs: {
            name: 'twitter:image:alt',
            content: 'Herdr documentation — One terminal. The whole herd.',
          },
        },
      ],
      editLink: {
        baseUrl: `${productGitHubUrl('/edit/main/')}`,
      },
      lastUpdated: true,
      disable404Route: true,
      sidebar: [
        {
          label: 'Start here',
          items: [
            { label: 'Overview', slug: 'docs' },
            { label: 'Install', slug: 'docs/install' },
            { label: 'Quick start', slug: 'docs/quick-start' },
            { label: 'Concepts', slug: 'docs/concepts' },
            { label: 'Keyboard', slug: 'docs/keyboard' },
          ],
        },
        {
          label: 'Using Herdr',
          items: [
            { label: 'How to work with Herdr', slug: 'docs/how-to-work' },
            { label: 'Agents', slug: 'docs/agents' },
            { label: 'Agent automation', slug: 'docs/agent-automation' },
            { label: 'Session state and restore', slug: 'docs/session-state' },
            { label: 'Persistence and remote access', slug: 'docs/persistence-remote' },
          ],
        },
        {
          label: 'Configure',
          items: [
            { label: 'Configuration', slug: 'docs/configuration' },
            { label: 'Config reference', slug: 'docs/config-reference' },
            { label: 'Plugins', slug: 'docs/plugins' },
            { label: 'Marketplace', slug: 'docs/marketplace' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'CLI reference', slug: 'docs/cli-reference' },
            { label: 'Socket API', slug: 'docs/socket-api' },
            { label: 'Integrations', slug: 'docs/integrations' },
            { label: 'Agent skill file', slug: 'docs/agent-skill' },
            { label: 'Windows beta', slug: 'docs/windows-beta' },
          ],
        },
        {
          label: 'Help',
          items: [
            { label: 'Troubleshooting', slug: 'docs/troubleshooting' },
            { label: 'Preview docs', slug: 'docs/preview' },
          ],
        },
      ],
    }),
  ],
  markdown: {
    remarkPlugins: [rewriteHerdrLinks],
  },
});
