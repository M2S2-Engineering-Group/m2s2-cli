import { Router } from 'express';

export const router = Router();

router.get('/info', (_req, res) => {
  res.json({ service: 'api', version: '1.0.0' });
});
