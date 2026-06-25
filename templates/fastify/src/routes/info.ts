import type { FastifyInstance } from 'fastify';

export async function infoRoutes(app: FastifyInstance) {
  app.get('/info', async () => ({ service: 'api', version: '1.0.0' }));
}
