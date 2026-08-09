import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';

const port = parseInt(process.env.PORT, 10) || 3000;

export default defineConfig({
  integrations: [tailwind()],
  output: 'static',
  server: { port, host: true },
  trailingSlash: 'always',
  devToolbar: { enabled: false },
});
