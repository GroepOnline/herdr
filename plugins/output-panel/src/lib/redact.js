/**
 * Secret redaction.
 *
 * Every byte a provider produces from a file, a JSON blob or a command is
 * pushed through here before it reaches the screen, an export file or stdout.
 * The rules are deliberately eager: a false positive costs a masked value, a
 * false negative leaks a credential.
 */

export const REDACTED = "[redacted]";

/** Key names whose value is never safe to print. */
const SECRET_KEY_RE =
  /(?:^|[_.-])(?:pass(?:word|wd|phrase)?|secret|secrets|token|tokens|api[_-]?key|apikey|access[_-]?key|authorization|auth[_-]?(?:token|key|secret|header)|credential|credentials|bearer|private[_-]?key|privatekey|client[_-]?secret|session[_-]?token|cookie|cookies|salt|signature|refresh[_-]?token|webhook[_-]?secret)(?:$|[_.-])/i;

/** Object keys whose entire subtree is environment/header material. */
const OPAQUE_CONTAINER_KEYS = /^(?:env|environment|headers|secrets|credentials)$/i;

/** Well-known credential shapes, redacted wherever they appear in free text. */
const TOKEN_PATTERNS = [
  /\bgh[pousr]_[A-Za-z0-9]{16,}/g, // GitHub
  /\bgithub_pat_[A-Za-z0-9_]{20,}/g,
  /\bglpat-[A-Za-z0-9_-]{16,}/g, // GitLab
  /\bsk-[A-Za-z0-9_-]{16,}/g, // OpenAI-style
  /\bxox[abprs]-[A-Za-z0-9-]{10,}/g, // Slack
  /\blin_(?:api|oauth)_[A-Za-z0-9]{16,}/g, // Linear
  /\bntn_[A-Za-z0-9]{20,}/g, // Notion
  /\bre_[A-Za-z0-9_]{20,}/g, // Resend
  /\btskey-[A-Za-z0-9-]{12,}/g, // Tailscale
  /\bAKIA[0-9A-Z]{16}\b/g, // AWS access key id
  /\bAIza[0-9A-Za-z_-]{30,}/g, // Google
  /\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{4,}/g, // JWT
];

/** `KEY=value`, `KEY: value`, `"KEY": "value"` where KEY looks secret. */
const ASSIGNMENT_RE =
  /(["']?)([A-Za-z0-9_.-]*(?:pass(?:word|wd|phrase)?|secret|token|api[_-]?key|apikey|access[_-]?key|authorization|credential|bearer|private[_-]?key|client[_-]?secret|cookie)[A-Za-z0-9_.-]*)\1(\s*[:=]\s*)(["']?)([^\s"',;}\]]+)\4/gi;

/** Credentials embedded in URLs and query strings. */
const URL_CREDS_RE = /([a-z][a-z0-9+.-]*:\/\/)([^/\s:@]+):([^/\s@]+)@/gi;
const QUERY_SECRET_RE =
  /([?&](?:access_token|api_key|apikey|auth|code|key|password|secret|sig|signature|token)=)([^&\s"']+)/gi;

/** `Authorization: Bearer xyz` / `Basic xyz` headers in raw log text. */
const HEADER_AUTH_RE = /\b(Bearer|Basic)\s+([A-Za-z0-9._~+/=-]{12,})/g;

/** Redact secrets out of arbitrary text. Safe to call on huge strings. */
export function redactText(text) {
  if (!text) return text;
  let out = String(text);
  out = out.replace(URL_CREDS_RE, (_m, scheme, user) => `${scheme}${user}:${REDACTED}@`);
  out = out.replace(QUERY_SECRET_RE, (_m, prefix) => `${prefix}${REDACTED}`);
  out = out.replace(HEADER_AUTH_RE, (_m, scheme) => `${scheme} ${REDACTED}`);
  out = out.replace(
    ASSIGNMENT_RE,
    (_m, q1, key, sep, q2) => `${q1}${key}${q1}${sep}${q2}${REDACTED}${q2}`,
  );
  for (const pattern of TOKEN_PATTERNS) {
    out = out.replace(pattern, REDACTED);
  }
  return out;
}

/** True when a value should be masked purely because of its key name. */
export function isSecretKey(key) {
  if (!key) return false;
  return SECRET_KEY_RE.test(String(key)) || /(?:_KEY|_TOKEN|_SECRET|_PASSWORD)$/i.test(String(key));
}

/**
 * Deep-copy a parsed JSON value with secrets masked.
 *
 * Keys named `env`/`headers`/`credentials` keep their structure (so you can
 * see which variables a server is configured with) but lose every value.
 */
export function redactJson(value, key = "", opaque = false) {
  if (value === null || value === undefined) return value;
  if (Array.isArray(value)) {
    return value.map((item) => redactJson(item, key, opaque));
  }
  if (typeof value === "object") {
    const out = {};
    for (const [k, v] of Object.entries(value)) {
      const childOpaque = opaque || OPAQUE_CONTAINER_KEYS.test(k);
      out[k] = redactJson(v, k, childOpaque);
    }
    return out;
  }
  if (typeof value === "string") {
    if (opaque && value.length > 0) return REDACTED;
    if (isSecretKey(key)) return REDACTED;
    return redactText(value);
  }
  if (opaque && typeof value === "number") return REDACTED;
  if (isSecretKey(key) && typeof value === "number") return REDACTED;
  return value;
}

/** Pretty-print a JSON value with redaction applied. */
export function redactedJsonString(value, indent = 2) {
  return JSON.stringify(redactJson(value), null, indent);
}
