import fetch from "node-fetch";
import FormData from "form-data";

const MEDIA_SERVICE_URL =
  process.env.MEDIA_SERVICE_URL || "http://media-svc:3010";

export interface UploadedLogo {
  fileId: string;
  url: string;
}

/**
 * Download a logo image from a remote URL and upload it to the media service.
 *
 * Expected Behavior:
 *   Fetches the logo bytes from `logoUrl`. Wraps them in a form-data
 *   multipart upload to `POST /api/media/upload` on the media service.
 *   Forwards the provided `authToken` in the Authorization header.
 *   Returns the media service's file_id and public URL.
 *
 *   Uses `node-fetch@2` and `form-data` for reliable multipart streaming
 *   compatibility with Actix-web / actix-multipart on the media service.
 *
 * Raises:
 *   Throws if the logo cannot be fetched, if the media service is
 *   unreachable, or if the media service returns a non-2xx response.
 *
 * Side Effects:
 *   - Makes an outbound HTTP GET request to `logoUrl` (network I/O).
 *   - Makes an outbound HTTP POST request to the media service (network I/O).
 */
export async function downloadAndUploadLogo(
  logoUrl: string,
  authToken: string
): Promise<UploadedLogo> {
  // Fetch logo bytes
  const logoResp = await fetch(logoUrl, {
    headers: {
      "User-Agent":
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    },
  });
  if (!logoResp.ok) {
    throw new Error(
      `Failed to fetch logo: ${logoResp.status} ${logoResp.statusText}`
    );
  }

  const contentType =
    logoResp.headers.get("content-type") || "image/png";
  const buffer = await logoResp.buffer();

  // Build multipart form using form-data (actix-multipart compatible)
  const form = new FormData();
  form.append("file", buffer, {
    filename: "logo.png",
    contentType,
  });
  form.append("folder", "brand-logos");

  const uploadResp = await fetch(`${MEDIA_SERVICE_URL}/api/media/upload`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${authToken}`,
      ...form.getHeaders(),
    },
    body: form,
  });

  if (!uploadResp.ok) {
    const body = await uploadResp.text();
    throw new Error(
      `Media upload failed: ${uploadResp.status} ${uploadResp.statusText} — ${body}`
    );
  }

  const data = (await uploadResp.json()) as {
    id: string;
    url?: string;
  };

  return {
    fileId: data.id,
    url: data.url || `${MEDIA_SERVICE_URL}/api/media/${data.id}`,
  };
}
