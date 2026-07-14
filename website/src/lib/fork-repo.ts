/** OnlineChefGroep downstream distribution repo (not upstream ogulcancelik/herdr). */
export const FORK_GITHUB_REPO = "OnlineChefGroep/herdr";

export function forkGitHubUrl(path = ""): string {
	const suffix = path.startsWith("/") ? path : path ? `/${path}` : "";
	return `https://github.com/${FORK_GITHUB_REPO}${suffix}`;
}

export const FORK_GITHUB_API_URL = `https://api.github.com/repos/${FORK_GITHUB_REPO}`;
