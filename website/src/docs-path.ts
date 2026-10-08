const docsVersionPattern = /^\d+\.\d+\.\d+$/;

export type DocsTarget = 'stable' | 'preview' | string;

export interface DocsRoute {
  isDocs: boolean;
  target: DocsTarget;
  page: string;
}

export interface DocsScope {
  locales: Record<string, string[]>;
}

export function docsRoute(pathname: string): DocsRoute {
  const segments = pathname.split('/').filter(Boolean);
  if (segments.shift() !== 'docs') {
    return { isDocs: false, target: 'stable', page: '' };
  }

  let target: DocsTarget = 'stable';
  if (segments[0] === 'preview') {
    target = 'preview';
    segments.shift();
  } else if (segments[0] && docsVersionPattern.test(segments[0])) {
    target = segments.shift()!;
  }

  return {
    isDocs: true,
    target,
    page: segments.join('/'),
  };
}

export function docsVersion(pathname: string) {
  return docsRoute(pathname).target;
}

export function docsTargetHref(
  route: DocsRoute,
  target: DocsTarget,
  scopes?: Record<string, DocsScope>,
) {
  if (!route.isDocs) throw new Error('cannot build a documentation target from a non-docs route');

  const availablePages = scopes?.[target]?.locales.root;
  const page = availablePages?.includes(route.page) === false ? '' : route.page;
  const targetSegment = target === 'stable' ? '' : `/${target}`;
  return `/docs${targetSegment}${page ? `/${page}` : ''}/`;
}

export function docsPath({ entry }: { entry: string }) {
  const slug = entry.replace(/\.(md|mdx|markdown|mdown|mkdn|mkd|mdwn)$/i, '');
  const normalized = slug.replace(/\/index$/, '');
  const segments = normalized.split('/');

  let targetSegment = '';
  if (segments[0] === 'preview') {
    targetSegment = 'preview';
    segments.shift();
  } else if (segments[0] === '_versions' && segments[1] && docsVersionPattern.test(segments[1])) {
    segments.shift();
    targetSegment = segments.shift()!;
  }

  const page = segments.join('/');
  const target = targetSegment ? `/${targetSegment}` : '';
  return `docs${target}${page && page !== 'index' ? `/${page}` : ''}`;
}
