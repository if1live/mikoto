import { expect } from 'expect';
import { VERSION } from '../src/index.js';

describe('foo', () => {
	it('bar', () => expect(VERSION).toBe('esm-minimal/0.0.1'));
});
