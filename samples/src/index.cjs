const AWS = require('aws-sdk');

const region = process.env.AWS_REGION || 'us-west-1';
const stage = process.env.STAGE || 'dev';

const sqs = new AWS.SQS();

/**
 * @param {*} event
 * @param {Context} context
 */
module.exports.sqs_enqueue = async (event, context) => {
	// "invokedFunctionArn": "arn:aws:lambda:ap-northeast-1:xxxx:function:xxxx"
	const arnTokens = context.invokedFunctionArn.split(':');
	const accountId = arnTokens[4];

	// TODO: 계정 이름 교체
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
 * @param {*} context
 * @link https://docs.aws.amazon.com/ko_kr/lambda/latest/dg/with-mq.html
 */
module.exports.common_dequeue = async (event, context) => {
	console.log('event', JSON.stringify(event, null, 2));
	console.log('context', JSON.stringify(context, null, 2));
}
