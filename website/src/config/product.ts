/** Canonical OnlineChefGroep Herdr product metadata (single source of truth). */
export const PRODUCT_GITHUB_ORG = "OnlineChefGroep";
export const PRODUCT_GITHUB_REPO = "OnlineChefGroep/herdr";
export const UPSTREAM_GITHUB_REPO = "ogulcancelik/herdr";
export const PRODUCT_SITE_URL = "https://herdr.chefgroep.nl";
export const PRODUCT_CONTACT_EMAIL = "hey@chefgroep.online";

export const productGitHubUrl = (path = ""): string => {
	const suffix = path.startsWith("/") ? path : path ? `/${path}` : "";
	return `https://github.com/${PRODUCT_GITHUB_REPO}${suffix}`;
};

export const productSiteUrl = (path = ""): string => {
	const suffix = path.startsWith("/") ? path : path ? `/${path}` : "";
	return `${PRODUCT_SITE_URL}${suffix}`;
};
