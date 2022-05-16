import { expect } from 'expect';
import { VERSION } from '../src/index.mjs';

describe('foo', () => {
	it('bar', () => expect(VERSION).toBe('esm-minimal/0.0.1'));
});
