// @ts-check
import { defineConfig } from "astro/config";

import node from "@astrojs/node";

// https://astro.build/config
export default defineConfig({
  output: "server",

  adapter: node({
    mode: "standalone"
  }),

  // this is actually so cringe but im going to go insane
  security: {
    checkOrigin: false
  }
});
