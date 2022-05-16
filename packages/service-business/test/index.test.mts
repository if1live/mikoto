import { expect } from 'expect';
import { add } from '../src/index.js';

describe('blank', () => {
	it('empty', () => expect(1).toBe(1));
	it('add', () => expect(add(1, 2)).toBe(3));
});
