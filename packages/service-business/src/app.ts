import { App, Request, Response } from '@tinyhttp/app';
import * as bodyParser from 'milliparsec';
import { redis, knex } from './instances.js';

export function initApp(): App {
	const app = new App();
	app.use(bodyParser.json());
	return app;
}

export const app = new App();
app.use(bodyParser.json());

export const subApp_hello = new App();

subApp_hello.get('/redis', async (req: Request, res: Response) => {
	const result = await redis.info();
	res.json(result);
});

subApp_hello.get('/knex', async (req: Request, res: Response) => {
	const rows = await knex.select(
		knex.raw('?', [knex.fn.now()]),
	);
	res.json(rows);
});

subApp_hello.use('/', async (req: Request, res: Response) => {
	const sls = (req as any).serverless;
	const data = {
		req: {
			method: req.method,
			path: req.path,
			params: req.params,
			query: req.query,
			body: req.body,
			headers: req.headers,
			ips: req.ips,
		},
		sls,
	};
	res
		.status(200)
		.type('application/json')
		.send(JSON.stringify(data, null, 2));
});

export const subApp_healthcheck = new App();
subApp_healthcheck.use('/', async (req: Request, res: Response) => {
	res.json({ ok: true });
});

app.use('/hello', subApp_hello);
app.use('/healthcheck', subApp_healthcheck);
