import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        block: resolve(__dirname, 'block.html'),
        tx: resolve(__dirname, 'tx.html'),
        address: resolve(__dirname, 'address.html'),
        network: resolve(__dirname, 'network.html'),
        supply: resolve(__dirname, 'supply.html'),
        peers: resolve(__dirname, 'peers.html'),
        nodes: resolve(__dirname, 'nodes.html'),
      },
    },
  },
});
