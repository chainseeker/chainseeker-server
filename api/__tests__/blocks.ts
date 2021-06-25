
import { getCS } from './common.lib';

const cs = getCS();

test('get blocks', async () => {
	const heights: number[] = [];
	for(let h=0; h<10; h++) {
		heights.push(h);
	}
	const blocks = await cs.getBlocks(heights);
	expect(blocks.length).toBe(10);
});
