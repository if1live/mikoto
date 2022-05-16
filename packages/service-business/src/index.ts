import {
	APIGatewayProxyHandlerV2,
} from 'aws-lambda';
import serverless from 'serverless-http';
import { VERSION } from '@mikoto/esm-minimal';
import * as ws from './ws.js';
import {
	app,
	subApp_healthcheck,
	subApp_hello,
	initApp,
} from './app.js';

// console.log('import.meta.url', import.meta.url);
console.log('VERSION', VERSION);

let handler_hello: serverless.Handler;
export const http_hello: APIGatewayProxyHandlerV2 = async (event, context) => {
	if (!handler_hello) {
		const app = initApp();
		app.use('/hello', subApp_hello);

		handler_hello = serverless(app.handler.bind(app), {
			request: (request: any) => {
				request.serverless = { event, context };
			},
		});
	}
	const result = await handler_hello(event, context);
	return result;
};

let handler_healthcheck: serverless.Handler;
export const http_healthcheck: APIGatewayProxyHandlerV2 = async (event, context) => {
	if (!handler_healthcheck) {
		const app = initApp();
		app.use('/healthcheck', subApp_healthcheck);

		handler_healthcheck = serverless(app.handler.bind(app), {
			request: (request: any) => {
				request.serverless = { event, context };
			},
		});
	}

	const result = await handler_healthcheck(event, context);
	return result;
};

export function add(a: number, b: number): number {
	return a + b;
}

export const ws_connect = ws.websocket_connect;
export const ws_disconnect = ws.websocket_disconnect;
export const ws_dispatch = ws.websocket_dispatch;

// TODO: 람다 감지는 다른 방식으로
/*
if (!process.env.LAMBDA_TASK_ROOT) {
	app.listen(3000, () => {
		console.log('server listening on 3000');
	});
}
*/

