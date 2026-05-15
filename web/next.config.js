/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  async rewrites() {
    const gatewayUrl = process.env.GATEWAY_INTERNAL_URL || 'http://gateway-svc:8000'
    return [
      {
        source: '/api/:path*',
        destination: `${gatewayUrl}/api/:path*`,
      },
    ]
  },
}

module.exports = nextConfig
