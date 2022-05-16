import {
	APIGatewayProxyHandler,
} from 'aws-lambda';
import {
	ApiGatewayManagementApi,
	PostToConnectionCommand,
} from '@aws-sdk/client-apigatewaymanagementapi';
import {
	DynamoDB,
	PutItemCommand,
	DeleteItemCommand,
	ScanCommand,
} from '@aws-sdk/client-dynamodb';

export function isNonNullable<T>(
	x: T | null | undefined,
): x is NonNullable<typeof x> {
	if (x === undefined) { return false; }
	if (x == null) { return false; }
	return true;
}

const region: string = process.env.AWS_REGION!;

const ddb = new DynamoDB({
	region,
});

export const websocket_connect: APIGatewayProxyHandler = async (event, context) => {
	const connectionId = event.requestContext.connectionId!;
	console.log({ _tag: 'ws_connect', connectionId });

	const command = new PutItemCommand({
		TableName: 'ConnectionIds',
		Item: {
			connectionId: { S: connectionId },
		},
	});

	try {
		// TODO: DynamoDBClient.send?
		await ddb.putItem(command.input);
		return {
			statusCode: 200,
			body: 'OK',
		}

	} catch (e) {
		console.error(e);
		const err = e as any;
		return {
			statusCode: 400,
			body: `${err.name}: ${err.message}`,
		};
	}
};

export const websocket_disconnect: APIGatewayProxyHandler = async (event, context) => {
	const connectionId = event.requestContext.connectionId!;
	console.log({ _tag: 'ws_disconnect', connectionId });

	const command = new DeleteItemCommand({
		TableName: 'ConnectionIds',
		Key: {
			connectionId: { S: connectionId },
		},
	});

	try {
		await ddb.deleteItem(command.input);
		return {
			statusCode: 200,
			body: 'OK',
		}

	} catch (e) {
		console.error(e);
		const err = e as any;
		return {
			statusCode: 400,
			body: `${err.name}: ${err.message}`,
		};
	}
};

export const websocket_dispatch: APIGatewayProxyHandler = async (event, context) => {
	const text = event.body ?? '<blank>';
	const encoder = new TextEncoder();
	const data = encoder.encode(text);

	const command = new ScanCommand({
		TableName: 'ConnectionIds',
		ProjectionExpression: 'connectionId',
	});

	// lambda: f3w1jmmhb3.execute-api.ap-northeast-2.amazonaws.com/dev
	// offline: private.execute-api.ap-northeast-2.amazonaws.com/local
	const apiId = event.requestContext.apiId!;
	const stage = event.requestContext.stage!;
	const endpoint = apiId === 'private'
		? `http://${(event.headers as any).Host}`
		: `https://${apiId}.execute-api.${region}.amazonaws.com/${stage}`;

	try {
		const dbResult = await ddb.scan(command.input);
		const connectionIds = (dbResult.Items ?? []).map(data => {
			return data.connectionId.S!;
		});

		const tasks = connectionIds.map(async connectionId => {
			const command = new PostToConnectionCommand({
				ConnectionId: connectionId,
				Data: data,
			});
			const apigw = new ApiGatewayManagementApi({
				endpoint,
			});
			return await apigw.postToConnection(command.input);
		});

		const results = await Promise.allSettled(tasks);
		const results_fulfilled = results.map(x => x.status === 'fulfilled' ? x : undefined).filter(isNonNullable);
		const results_rejected = results.map(x => x.status === 'rejected' ? x : undefined).filter(isNonNullable);

		console.log({
			_tag: 'ws_broadcast',
			count_fulfilled: results_fulfilled.length,
			count_rejected: results_rejected.length,
		});

		return {
			statusCode: 200,
			body: 'OK',
		}

	} catch (e) {
		console.error(e);
		const err = e as any;
		return {
			statusCode: 400,
			body: `${err.name}: ${err.message}`,
		};
	}
};
