const AWS = require('aws-sdk');
const amqplib = require('amqplib');
const { setTimeout } = require('timers/promises');

const region = process.env.AWS_REGION || 'ap-northeast-1';
const stage = process.env.STAGE || 'dev';

const RABBITMQ_URI = process.env.RABBITMQ_URI || "amqp://guest:guest@127.0.0.1";

const sqs = new AWS.SQS();

/**
 * @param {*} event
 * @param {Context} context
 */
module.exports.sqs_enqueue = async (event, context) => {
	// "invokedFunctionArn": "arn:aws:lambda:ap-northeast-1:xxxx:function:xxxx"
	const arnTokens = context.invokedFunctionArn.split(':');
	const accountId = arnTokens[4];

	const queueUrl = `https://sqs.${region}.amazonaws.com/${accountId}/mikoto-sample-${stage}-demo`;

	const now = new Date();
	const body = {
		now,
		event,
		context,
	};

	try {
		const result = await sqs.sendMessage({
			QueueUrl: queueUrl,
			DelaySeconds: 5,
			MessageBody: JSON.stringify(body),
		}).promise();
		console.log('ok', JSON.stringify(result));

		return {
			ok: true,
			result,
		};

	} catch (e) {
		console.error('error', e);
		return {
			ok: false,
			error: {
				name: e.name,
				message: e.message,
				stack: e.stack,
			},
		};
	}
}

/**
 * @param {*} event
 * @param {Context} context
 */
module.exports.amqp_enqueue = async (event, context) => {
	const queue = 'hello';
	const message_text = `${Date.now()}`;
	const message_buffer = Buffer.from(message_text);

	let connection = null;
	try {
		connection = await amqplib.connect(RABBITMQ_URI);

		const channel = await connection.createChannel();
		await channel.assertQueue(queue, {
			durable: false
		});

		const result = channel.sendToQueue(queue, message_buffer);
		// sendToQueue 이후에 delay를 넣어서 메세지를 보내기
		await setTimeout(10);

		return {
			ok: true,
			result: result,
		};

	} catch (e) {
		console.error('error', e);
		return {
			ok: false,
			error: {
				name: e.name,
				message: e.message,
				stack: e.stack,
			},
		};

	} finally {
		if (connection) {
			await connection.close();
		}
	}
};

/**
 * @param {*} event
 * @param {*} context
 * @link https://docs.aws.amazon.com/ko_kr/lambda/latest/dg/with-mq.html
 */
module.exports.common_dequeue = async (event, context) => {
	console.log('event', JSON.stringify(event, null, 2));
	console.log('context', JSON.stringify(context, null, 2));
}
