import {
	PRODUCT_GITHUB_REPO,
	productGitHubUrl,
} from "../config/product.ts";

/** OnlineChefGroep downstream distribution repo (not upstream ogulcancelik/herdr). */
export const FORK_GITHUB_REPO = PRODUCT_GITHUB_REPO;

export function forkGitHubUrl(path = ""): string {
	return productGitHubUrl(path);
}

export const FORK_GITHUB_API_URL = `https://api.github.com/repos/${FORK_GITHUB_REPO}`;
