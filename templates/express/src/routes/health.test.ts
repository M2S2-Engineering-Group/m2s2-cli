import { describe, expect, it } from 'vitest';
import express from 'express';
import http from 'node:http';
import type { AddressInfo } from 'node:net';
import { router } from './health.js';

function get(path: string, port: number): Promise<{ status: number; body: unknown }> {
  return new Promise((resolve, reject) => {
    http
      .get({ host: '127.0.0.1', port, path }, (res) => {
        let data = '';
        res.on('data', (chunk) => {
          data += chunk;
        });
        res.on('end', () => {
          resolve({ status: res.statusCode ?? 0, body: JSON.parse(data) });
        });
      })
      .on('error', reject);
  });
}

describe('GET /health', () => {
  it('returns ok', async () => {
    const app = express();
    app.use(router);
    const server = app.listen(0);
    await new Promise<void>((resolve) => server.once('listening', resolve));
    const { port } = server.address() as AddressInfo;

    try {
      const { status, body } = await get('/health', port);
      expect(status).toBe(200);
      expect(body).toEqual({ status: 'ok' });
    } finally {
      server.close();
    }
  });
});
