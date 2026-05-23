import fastify from "fastify";
import { scrapeBrand } from "./scraper";
import { downloadAndUploadLogo } from "./media";
import { normalizeBrand } from "./ai";

const PORT = parseInt(process.env.PORT || "3012", 10);

const app = fastify({ logger: true });

/**
 * Extract brand assets from a website and normalize them into a theme.
 *
 * Expected Behavior:
 *   Receives a POST request with `{ url: string }` and a Bearer token in
 *   the Authorization header. Uses a transient Playwright browser to scrape
 *   the target website for logo, colors, and fonts. Downloads the logo and
 *   uploads it to the media service for permanent storage. Sends the raw
 *   scraped data to the AI service (`/api/ai/brand-normalize`) to get
 *   DaisyUI semantic color mappings, a suggested preset, and font
 *   recommendations. Returns a combined response.
 *
 *   If the logo cannot be downloaded or uploaded, the response still
 *   succeeds but `logoUrl` will be the external URL (or null).
 *
 * Raises:
 *   Returns 400 if the URL is missing/invalid.
 *   Returns 401 if the Authorization header is missing.
 *   Returns 502 if scraping fails or the AI/media services are unreachable.
 *
 * Side Effects:
 *   - Launches a headless Chromium browser per request (CPU + memory).
 *   - Downloads the logo image from the external URL (network I/O).
 *   - Uploads the logo to the internal media service (network I/O).
 *   - Calls the internal AI service for normalization (network I/O).
 */
app.post("/api/ai/brand-extract", async (request, reply) => {
  const authHeader = request.headers.authorization;
  if (!authHeader || !authHeader.startsWith("Bearer ")) {
    reply.status(401);
    return { error: "Missing or invalid Authorization header" };
  }
  const authToken = authHeader.slice(7);

  const body = request.body as { url?: string };
  if (!body.url) {
    reply.status(400);
    return { error: "Missing required field: url" };
  }

  let targetUrl: string;
  try {
    targetUrl = new URL(body.url).href;
  } catch {
    reply.status(400);
    return { error: "Invalid URL" };
  }

  try {
    // 1. Scrape
    const scraped = await scrapeBrand(targetUrl);

    // 2. Download & upload logo (best effort)
    let logoUrl: string | null = scraped.logoUrl;
    if (scraped.logoUrl) {
      try {
        const uploaded = await downloadAndUploadLogo(scraped.logoUrl, authToken);
        logoUrl = uploaded.url;
      } catch (logoErr) {
        app.log.warn({ err: logoErr }, "Logo upload failed; keeping external URL");
      }
    }

    // 3. Normalize via AI service
    const normalized = await normalizeBrand(
      scraped.rawColors,
      scraped.rawFonts,
      scraped.websiteVibe,
      authToken,
      scraped.colorContext
    );

    return {
      logoUrl,
      semanticColors: normalized.semanticColors,
      suggestedPreset: normalized.suggestedPreset,
      fontConfig: normalized.fontConfig,
      extracted: {
        rawColors: scraped.rawColors,
        rawFonts: scraped.rawFonts,
        websiteVibe: scraped.websiteVibe,
      },
    };
  } catch (err: any) {
    app.log.error({ err }, "Brand extraction failed");
    reply.status(502);
    return {
      error: "Brand extraction failed",
      message: err.message || "Unknown error",
    };
  }
});

/**
 * Health check endpoint.
 *
 * Expected Behavior:
 *   Returns 200 with status "healthy". Used by Docker health checks and
 *   the gateway composite health check.
 *
 * Side Effects:
 *   None.
 */
app.get("/health", async () => {
  return { status: "healthy", service: "ai-scraper" };
});

/**
 * Start the HTTP server.
 *
 * Expected Behavior:
 *   Binds to 0.0.0.0 on the configured port and begins accepting requests.
 *   Logs startup information.
 *
 * Side Effects:
 *   - Binds a TCP port.
 *   - Starts the Fastify event loop.
 */
async function start() {
  try {
    await app.listen({ port: PORT, host: "0.0.0.0" });
    app.log.info(`AI Scraper service listening on port ${PORT}`);
  } catch (err) {
    app.log.error(err);
    process.exit(1);
  }
}

start();
