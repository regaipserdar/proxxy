import { TrafficRequest } from "@/components/traffic/types-grapql";

export const formatRequestRaw = (req: TrafficRequest): string => {
    let url;
    try {
        url = new URL(req.url);
    } catch {
        // Fallback for URLs without protocol (e.g. "example.com/path" or "host:port")
        if (req.method === 'CONNECT') {
            return `CONNECT ${req.url} HTTP/1.1\nHost: ${req.url.split(':')[0]}\n\n`;
        }
        return `${req.method} ${req.url} HTTP/1.1\n\n`;
    }

    const path = url.pathname + url.search;
    let raw = `${req.method} ${path || '/'} HTTP/1.1\n`;
    raw += `Host: ${url.host}\n`;

    if (req.requestHeaders) {
        try {
            const headers = JSON.parse(req.requestHeaders);
            // Handle both flat object and { headers: {...} } format
            const entries = headers.headers || headers;
            Object.entries(entries).forEach(([k, v]) => {
                if (k.toLowerCase() !== 'host') {
                    raw += `${k}: ${v}\n`;
                }
            });
        } catch (e) {
            console.error('[HTTP-Utils] Failed to parse headers:', e);
        }
    }

    raw += '\n';
    if (req.requestBody) {
        // If it's a string, just append it.
        // TODO: Handle binary/base64 if orchestrator sends it that way.
        raw += req.requestBody;
    }

    return raw;
};

/**
 * Parses a raw HTTP request string into components
 */
export const parseRawRequest = (raw: string) => {
    // Normalize line endings
    const normalized = raw.replace(/\r\n/g, '\n');
    const [headerPart, ...bodyParts] = normalized.split('\n\n');
    const lines = headerPart.split('\n');

    if (lines.length === 0) return null;

    const firstLine = lines[0];
    const parts = firstLine.split(' ');
    if (parts.length < 2) return null;

    const method = parts[0];
    const path = parts[1];

    const headers: Record<string, string> = {};
    for (let i = 1; i < lines.length; i++) {
        const line = lines[i];
        const colonIndex = line.indexOf(':');
        if (colonIndex !== -1) {
            const key = line.slice(0, colonIndex).trim();
            const value = line.slice(colonIndex + 1).trim();
            headers[key] = value;
        }
    }

    // Construct full URL if it's just a path
    let url = path;
    const host = headers['Host'] || headers['host'];
    if (path.startsWith('/') && host) {
        // Default to https since most security testing targets use TLS
        // The agent's reqwest client supports both http and https
        url = `https://${host}${path}`;
    }

    return {
        method,
        url,
        headers: JSON.stringify(headers),
        body: bodyParts.join('\n\n')
    };
};
