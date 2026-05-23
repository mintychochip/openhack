const AI_SERVICE_URL =
  process.env.AI_SERVICE_URL || "http://ai-svc:3007";

export interface ColorContext {
  heroColors: string[];
  ctaColors: string[];
  headingColors: string[];
  dominantColors: { hex: string; count: number }[];
}

export interface NormalizedBrand {
  semanticColors: Record<string, string | null>;
  suggestedPreset: string;
  fontConfig: {
    display?: { url: string; family: string };
    heading?: { url: string; family: string };
    body?: { url: string; family: string };
  };
}

/**
 * Send raw scraped brand data to the AI service for normalization.
 *
 * Expected Behavior:
 *   Calls `POST /api/ai/brand-normalize` on the AI service with the raw
 *   colors, fonts, and vibe. Forwards the provided `authToken` in the
 *   Authorization header. Returns the normalized theme data (semantic
 *   colors, suggested DaisyUI preset, and font recommendations).
 *
 * Raises:
 *   Throws if the AI service is unreachable, returns a non-2xx response,
 *   or if the response body cannot be parsed.
 *
 * Side Effects:
 *   - Makes an outbound HTTP POST request to the AI service (network I/O).
 */
export async function normalizeBrand(
  rawColors: string[],
  rawFonts: string[],
  websiteVibe: string,
  authToken: string,
  colorContext?: ColorContext
): Promise<NormalizedBrand> {
  const resp = await fetch(`${AI_SERVICE_URL}/api/ai/brand-normalize`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${authToken}`,
    },
    body: JSON.stringify({
      raw_colors: rawColors,
      raw_fonts: rawFonts,
      website_vibe: websiteVibe,
      color_context: colorContext ? {
        hero_colors: colorContext.heroColors,
        cta_colors: colorContext.ctaColors,
        heading_colors: colorContext.headingColors,
        dominant_colors: colorContext.dominantColors,
      } : undefined,
    }),
  });

  if (!resp.ok) {
    const body = await resp.text();
    throw new Error(
      `AI normalization failed: ${resp.status} ${resp.statusText} — ${body}`
    );
  }

  const data = (await resp.json()) as {
    semantic_colors: Record<string, string | null>;
    suggested_preset: string;
    font_config: {
      display?: { url: string; family: string };
      heading?: { url: string; family: string };
      body?: { url: string; family: string };
    };
  };

  return {
    semanticColors: data.semantic_colors,
    suggestedPreset: data.suggested_preset,
    fontConfig: data.font_config,
  };
}
