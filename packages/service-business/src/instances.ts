import mysql from 'mysql2/promise';

import * as KnexModule from 'knex';
const knexInit = KnexModule.default.knex;

import RedisModule from 'ioredis';
const Redis = RedisModule.default;

if (process.env.npm_lifecycle_event === 'test') {
	process.env.NODE_ENV = 'test';
}

function openRedis(endpoint: string) {
	// URL이 https로 시작하지 않으면 암호쪽까지 제대로 파싱 안하는듯?
	const text = endpoint.replace('redis://', 'http://');
	const url = new URL(text);

	return new Redis({
		host: url.hostname,
		port: parseInt(url.port, 10),
		password: url.password,
		db: parseInt(url.pathname.substring(1), 10),
	});
}
export const redis: RedisModule.Redis = process.env.NODE_ENV !== 'test'
	? openRedis(process.env.REDIS_URI!)
	: {} as any;

function openKnex(endpoint: string) {
	// TODO: db engine 대응
	const text = endpoint.replace('mysql://', 'http://');
	const url = new URL(text);

	return knexInit({
		client: 'mysql2',
		connection: {
			host: url.hostname,
			port: parseInt(url.port, 10),
			user: url.username,
			password: url.password,
			database: url.pathname.substring(1),
		},
	});
}

// TODO: 모듈로 정의된 클라로는 유닛테스트 작성이 까다로움
export const knex: KnexModule.Knex = process.env.NODE_ENV !== 'test'
	? openKnex(process.env.MYSQL_URI!)
	: {} as any;
