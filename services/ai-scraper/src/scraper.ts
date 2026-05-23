import { chromium, Page } from "playwright";

export interface ScrapedBrand {
  logoUrl: string | null;
  rawColors: string[];
  rawFonts: string[];
  websiteVibe: string;
  colorContext: ColorContext;
}

export interface ColorContext {
  /** Colors found in the hero / above-the-fold region */
  heroColors: string[];
  /** Colors from primary CTA buttons */
  ctaColors: string[];
  /** Colors from heading text */
  headingColors: string[];
  /** All extracted colors sorted by visual prominence */
  dominantColors: { hex: string; count: number }[];
}

// ───────────────────────────────────────────────────────────────
// Public API
// ───────────────────────────────────────────────────────────────

export async function scrapeBrand(url: string): Promise<ScrapedBrand> {
  const browser = await chromium.launch();
  const context = await browser.newContext({
    userAgent:
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    viewport: { width: 1280, height: 900 },
  });
  const page = await context.newPage();

  try {
    await page.goto(url, { waitUntil: "networkidle", timeout: 30000 });
    // Wait a tick for any lazy CSS/hero images to settle
    await page.waitForTimeout(800);

    const [logoUrl, rawColors, rawFonts, websiteVibe, colorContext] = await Promise.all([
      extractLogo(page, url),
      extractDominantColors(page),
      extractFonts(page),
      deriveVibe(page),
      extractColorContext(page),
    ]);

    return {
      logoUrl,
      rawColors,
      rawFonts,
      websiteVibe,
      colorContext,
    };
  } finally {
    await context.close();
    await browser.close();
  }
}

// ───────────────────────────────────────────────────────────────
// Logo extraction
// ───────────────────────────────────────────────────────────────

async function extractLogo(page: Page, baseUrl: string): Promise<string | null> {
  const candidates = await page.evaluate(() => {
    const results: { url: string; area: number; isLogo: boolean }[] = [];

    // Helper to resolve relative URLs
    const resolve = (u: string) => {
      try {
        return new URL(u, window.location.href).href;
      } catch {
        return u;
      }
    };

    // 1. Meta tags (highest priority)
    const og = document.querySelector('meta[property="og:image"]') as HTMLMetaElement | null;
    if (og?.content) {
      results.push({ url: resolve(og.content), area: 10000, isLogo: false });
    }
    const tw = document.querySelector('meta[name="twitter:image"]') as HTMLMetaElement | null;
    if (tw?.content) {
      results.push({ url: resolve(tw.content), area: 10000, isLogo: false });
    }

    // 2. Structured data / JSON-LD
    const scripts = document.querySelectorAll('script[type="application/ld+json"]');
    Array.from(scripts).forEach((s) => {
      try {
        const data = JSON.parse(s.textContent || "{}");
        const logo = data.logo || data.image;
        if (typeof logo === "string") {
          results.push({ url: resolve(logo), area: 10000, isLogo: false });
        }
      } catch { /* ignore */ }
    });

    // 3. Any img with logo-related class / alt / id
    const logoImgs = document.querySelectorAll(
      'img[class*="logo" i], img[alt*="logo" i], img[id*="logo" i], ' +
      'img[class*="brand" i], img[alt*="brand" i], ' +
      'svg[class*="logo" i], svg[class*="brand" i], ' +
      '[class*="logo" i] img, [class*="brand" i] img'
    );
    Array.from(logoImgs).forEach((el) => {
      const img = el as HTMLImageElement;
      const rect = img.getBoundingClientRect();
      const area = rect.width * rect.height;
      if (img.src) {
        results.push({ url: resolve(img.src), area, isLogo: true });
      }
      // Check srcset
      if (img.srcset) {
        const firstSrc = img.srcset.split(",")[0]?.trim().split(" ")[0];
        if (firstSrc) {
          results.push({ url: resolve(firstSrc), area, isLogo: true });
        }
      }
      // Check CSS background-image on parent or self
      const style = window.getComputedStyle(el);
      const bg = style.backgroundImage;
      if (bg && bg !== "none") {
        const match = bg.match(/url\(["']?([^"')]+)["']?\)/);
        if (match) {
          results.push({ url: resolve(match[1]), area, isLogo: true });
        }
      }
    });

    // 4. Header / nav images (largest wins)
    const headerImgs = document.querySelectorAll("header img, nav img, [role='banner'] img");
    Array.from(headerImgs).forEach((el) => {
      const img = el as HTMLImageElement;
      if (img.src) {
        const rect = img.getBoundingClientRect();
        const area = rect.width * rect.height;
        results.push({ url: resolve(img.src), area, isLogo: false });
      }
    });

    // 5. Favicon (lowest priority)
    const icon = document.querySelector('link[rel~="icon"]') as HTMLLinkElement | null;
    if (icon?.href) {
      results.push({ url: resolve(icon.href), area: 64, isLogo: false });
    }

    return results;
  });

  // Filter: minimum 40×40px (area 1600) unless it's explicitly a logo meta tag
  const filtered = candidates.filter((c) => c.area >= 1600 || c.isLogo);

  // Sort: logo-class items first, then by area descending
  filtered.sort((a, b) => {
    if (a.isLogo && !b.isLogo) return -1;
    if (!a.isLogo && b.isLogo) return 1;
    return b.area - a.area;
  });

  if (filtered.length === 0) return null;

  // Take the best candidate
  const best = filtered[0];
  try {
    return new URL(best.url, baseUrl).href;
  } catch {
    return best.url;
  }
}

// ───────────────────────────────────────────────────────────────
// Color extraction — screenshot pixel analysis
// ───────────────────────────────────────────────────────────────

async function extractDominantColors(page: Page): Promise<string[]> {
  const colors = await analyzePageColorsViaScreenshot(page);
  return colors.map((c) => c.hex);
}

async function extractColorContext(page: Page): Promise<ColorContext> {
  const [heroColors, ctaColors, headingColors, dominantColors] = await Promise.all([
    extractHeroColors(page),
    extractCTAColors(page),
    extractHeadingColors(page),
    analyzePageColorsViaScreenshot(page),
  ]);

  return {
    heroColors,
    ctaColors,
    headingColors,
    dominantColors,
  };
}

async function analyzePageColorsViaScreenshot(page: Page): Promise<{ hex: string; count: number }[]> {
  const screenshot = await page.screenshot({ type: "png" });
  const dataUrl = `data:image/png;base64,${screenshot.toString("base64")}`;

  return page.evaluate((imageDataUrl) => {
    return new Promise<{ hex: string; count: number }[]>((resolve) => {
      const img = new Image();
      img.onload = () => {
        const canvas = document.createElement("canvas");
        const maxDim = 400; // Downsample for speed
        const scale = Math.min(maxDim / img.width, maxDim / img.height, 1);
        canvas.width = Math.round(img.width * scale);
        canvas.height = Math.round(img.height * scale);
        const ctx = canvas.getContext("2d", { willReadFrequently: true })!;
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);

        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
        const pixels = imageData.data;

        const colorMap = new Map<string, number>();
        const step = 4; // Sample every 4th pixel for performance

        for (let i = 0; i < pixels.length; i += step * 4) {
          const r = pixels[i];
          const g = pixels[i + 1];
          const b = pixels[i + 2];
          const a = pixels[i + 3];

          if (a < 128) continue; // Skip transparent
          if (isNearGray(r, g, b)) continue; // Skip grays
          if (isNearWhite(r, g, b)) continue; // Skip near-white
          if (isNearBlack(r, g, b)) continue; // Skip near-black

          // Quantize to reduce similar colors
          const qr = Math.round(r / 16) * 16;
          const qg = Math.round(g / 16) * 16;
          const qb = Math.round(b / 16) * 16;
          const hex = rgbToHex(qr, qg, qb);

          colorMap.set(hex, (colorMap.get(hex) || 0) + 1);
        }

        const sorted = Array.from(colorMap.entries())
          .map(([hex, count]) => ({ hex, count }))
          .sort((a, b) => b.count - a.count)
          .slice(0, 10);

        resolve(sorted);
      };
      img.src = imageDataUrl;
    });

    function rgbToHex(r: number, g: number, b: number): string {
      return "#" + [r, g, b].map((x) => x.toString(16).padStart(2, "0")).join("");
    }

    function isNearGray(r: number, g: number, b: number): boolean {
      const maxDiff = Math.max(r, g, b) - Math.min(r, g, b);
      return maxDiff < 20;
    }

    function isNearWhite(r: number, g: number, b: number): boolean {
      return r > 240 && g > 240 && b > 240;
    }

    function isNearBlack(r: number, g: number, b: number): boolean {
      return r < 20 && g < 20 && b < 20;
    }
  }, dataUrl);
}

async function extractHeroColors(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const colors = new Set<string>();
    const add = (c: string) => {
      if (!c || c === "rgba(0, 0, 0, 0)" || c === "transparent") return;
      const hex = rgbToHex(c);
      if (hex && hex !== "#000000" && !isNearWhiteOrGray(hex)) {
        colors.add(hex);
      }
    };

    // Hero sections: first large visible block
    const heroSelectors = [
      "header", "[class*='hero' i]", "[class*='banner' i]",
      "section:first-of-type", ".jumbotron", "[role='banner']",
      "[id*='hero' i]", "[id*='banner' i]",
    ];

    for (const sel of heroSelectors) {
      const el = document.querySelector(sel);
      if (el) {
        const style = getComputedStyle(el);
        add(style.backgroundColor);
        add(style.color);
        // Check for CSS gradient backgrounds (they won't show in backgroundColor)
        const bgImage = style.backgroundImage;
        if (bgImage && bgImage.includes("gradient")) {
          // Extract colors from gradient string
          const matches = bgImage.match(/#[0-9a-fA-F]{3,8}|rgb\([^)]+\)|rgba\([^)]+\)/g);
          if (matches) {
            matches.forEach(add);
          }
        }
      }
    }

    // Also check the first full-width div with a background
    const firstSection = document.querySelector("body > div:first-of-type, body > section:first-of-type, main > section:first-of-type");
    if (firstSection) {
      const style = getComputedStyle(firstSection);
      add(style.backgroundColor);
    }

    return Array.from(colors).slice(0, 5);

    function rgbToHex(cssColor: string): string | null {
      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d");
      if (!ctx) return null;
      ctx.fillStyle = cssColor;
      return ctx.fillStyle;
    }

    function isNearWhiteOrGray(hex: string): boolean {
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);
      if (r > 240 && g > 240 && b > 240) return true;
      const maxDiff = Math.max(r, g, b) - Math.min(r, g, b);
      return maxDiff < 15;
    }
  });
}

async function extractCTAColors(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const colors = new Set<string>();
    const add = (c: string) => {
      if (!c || c === "rgba(0, 0, 0, 0)" || c === "transparent") return;
      const hex = rgbToHex(c);
      if (hex && !isNearWhiteOrGray(hex)) colors.add(hex);
    };

    // CTA buttons: look for prominent buttons
    const ctaSelectors = [
      "button[class*='cta' i]", "a[class*='cta' i]",
      "button[class*='primary' i]", "a[class*='primary' i]",
      "button[class*='action' i]", "a[class*='action' i]",
      "[class*='btn-primary' i]", "[class*='button-primary' i]",
      "button", "a",
    ];

    ctaSelectors.forEach((sel) => {
      const els = document.querySelectorAll(sel);
      Array.from(els).forEach((el) => {
        const rect = el.getBoundingClientRect();
        // Only visible, reasonably-sized elements
        if (rect.width > 60 && rect.height > 20 && rect.top < 800) {
          const style = getComputedStyle(el);
          add(style.backgroundColor);
          add(style.color);
          add(style.borderColor);
        }
      });
    });

    return Array.from(colors).slice(0, 5);

    function rgbToHex(cssColor: string): string | null {
      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d");
      if (!ctx) return null;
      ctx.fillStyle = cssColor;
      return ctx.fillStyle;
    }

    function isNearWhiteOrGray(hex: string): boolean {
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);
      if (r > 240 && g > 240 && b > 240) return true;
      const maxDiff = Math.max(r, g, b) - Math.min(r, g, b);
      return maxDiff < 15;
    }
  });
}

async function extractHeadingColors(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const colors = new Set<string>();
    const add = (c: string) => {
      if (!c || c === "rgba(0, 0, 0, 0)" || c === "transparent") return;
      const hex = rgbToHex(c);
      if (hex && !isNearWhiteOrGray(hex) && hex !== "#000000") colors.add(hex);
    };

    const headings = document.querySelectorAll("h1, h2, h3");
    Array.from(headings).forEach((h) => {
      const rect = h.getBoundingClientRect();
      if (rect.top < 600) { // Only above-the-fold headings
        const style = getComputedStyle(h);
        add(style.color);
      }
    });

    return Array.from(colors).slice(0, 5);

    function rgbToHex(cssColor: string): string | null {
      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d");
      if (!ctx) return null;
      ctx.fillStyle = cssColor;
      return ctx.fillStyle;
    }

    function isNearWhiteOrGray(hex: string): boolean {
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);
      if (r > 240 && g > 240 && b > 240) return true;
      const maxDiff = Math.max(r, g, b) - Math.min(r, g, b);
      return maxDiff < 15;
    }
  });
}

// ───────────────────────────────────────────────────────────────
// Font extraction
// ───────────────────────────────────────────────────────────────

async function extractFonts(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const fonts = new Map<string, number>(); // font -> weight (how prominent)

    const scoreFont = (el: Element, weight: number) => {
      const f = getComputedStyle(el).fontFamily;
      if (!f) return;
      // Clean up: take first font before comma, remove quotes
      const first = f.split(",")[0].trim().replace(/^['"]|['"]$/g, "");
      if (first && first !== "serif" && first !== "sans-serif" && first !== "monospace") {
        fonts.set(first, (fonts.get(first) || 0) + weight);
      }
    };

    // Headings get higher weight
    document.querySelectorAll("h1").forEach((el) => scoreFont(el, 10));
    document.querySelectorAll("h2, h3").forEach((el) => scoreFont(el, 5));
    document.querySelectorAll("body, p").forEach((el) => scoreFont(el, 1));

    return Array.from(fonts.entries())
      .sort((a, b) => b[1] - a[1])
      .map(([font]) => font)
      .slice(0, 6);
  });
}

// ───────────────────────────────────────────────────────────────
// Vibe
// ───────────────────────────────────────────────────────────────

async function deriveVibe(page: Page): Promise<string> {
  const title = await page.title();
  const description = await page.evaluate(() => {
    const meta = document.querySelector('meta[name="description"]') as HTMLMetaElement | null;
    return meta?.content || "";
  });

  // Sample a few colors from the screenshot to describe warmth
  const dominant = await extractDominantColors(page);
  let warmCount = 0;
  let coolCount = 0;
  for (const hex of dominant.slice(0, 5)) {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    if (r > g && r > b) warmCount++;
    else if (b > r && b > g) coolCount++;
  }

  const warmth = warmCount > coolCount ? "warm" : coolCount > warmCount ? "cool" : "neutral";
  const tag = description ? ` — ${description.slice(0, 120)}` : "";
  return `${title.trim()} (${warmth} aesthetic)${tag}`;
}
