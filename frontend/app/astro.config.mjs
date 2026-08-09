import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';
import node from '@astrojs/node';
import articles from '@articles/frontend/integration';

const port = parseInt(process.env.PORT, 10) || 3000;

export default defineConfig({
  integrations: [
    tailwind(),
    articles(),
  ],
  output: 'server',
  adapter: node({ mode: 'standalone' }),
  server: { port, host: true },
  trailingSlash: 'always',
  devToolbar: { enabled: false },
});
