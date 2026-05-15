/**
 * QRCode - Displays a QR code using qr-server API.
 * 
 * Expected Behavior:
 *   Renders QR code image from value.
 *   Supports custom size and styling.
 * 
 * Args:
 *   value: String to encode in QR code.
 *   size: QR code size in pixels.
 * 
 * Returns:
 *   QR code image component.
 */
'use client';

interface QRCodeProps {
  value: string;
  size?: number;
  className?: string;
}

export function QRCode({ value, size = 256, className = "" }: QRCodeProps) {
  const qrUrl = `https://api.qrserver.com/v1/create-qr-code/?size=${size}x${size}&data=${encodeURIComponent(value)}`;

  return (
    <img
      src={qrUrl}
      alt="QR Code"
      width={size}
      height={size}
      className={className}
      loading="lazy"
    />
  );
}

/**
 * QRCodeDisplay - QR code with label and download button.
 * 
 * Expected Behavior:
 *   Displays QR code with title.
 *   Allows downloading as PNG.
 */
interface QRCodeDisplayProps {
  title: string;
  value: string;
  size?: number;
}

export function QRCodeDisplay({ title, value, size = 256 }: QRCodeDisplayProps) {
  const qrUrl = `https://api.qrserver.com/v1/create-qr-code/?size=${size}x${size}&data=${encodeURIComponent(value)}`;

  const handleDownload = async () => {
    try {
      const response = await fetch(qrUrl);
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `${title.replace(/\s+/g, '_')}_QR.png`;
      link.click();
      window.URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to download QR code:', error);
    }
  };

  return (
    <div className="flex flex-col items-center gap-4">
      <div className="bg-white p-4 rounded-lg border shadow-sm">
        <QRCode value={value} size={size} />
      </div>
      <div className="text-center">
        <p className="font-medium">{title}</p>
        <button
          onClick={handleDownload}
          className="text-sm text-primary hover:underline mt-2"
        >
          Download PNG
        </button>
      </div>
    </div>
  );
}
